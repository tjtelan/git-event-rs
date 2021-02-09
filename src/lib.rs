//use calloop::{generic::Generic, EventLoop, Interest, Mode};
use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use mktemp::Temp;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use log::{debug, info};

use git_meta::{GitCommitMeta, GitCredentials, GitRepo};

type BranchHeads = HashMap<String, GitCommitMeta>;

#[derive(Clone, Debug)]
pub struct GitRepoWatchHandler {
    pub repo: GitRepo,
    pub state: Option<GitRepoState>,
    pub branch_filter: Option<Vec<String>>,
    pub use_shallow: bool,
    pub poll_freq: Duration,
    // TODO:
    //path_filter: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitRepoState {
    pub last_updated: Option<DateTime<Utc>>,
    pub branch_heads: BranchHeads,
}

impl GitRepoWatchHandler {
    pub fn new<U: AsRef<str>>(url: U) -> Result<Self> {
        Ok(GitRepoWatchHandler {
            repo: GitRepo::new(url)?,
            state: None,
            branch_filter: None,
            use_shallow: false,
            poll_freq: Duration::from_secs(5),
        })
    }

    pub fn with_credentials(mut self, creds: Option<GitCredentials>) -> Self {
        self.repo.credentials = creds;
        self
    }

    pub fn with_branch_filter(mut self, branch_list: Option<Vec<String>>) -> Self {
        self.branch_filter = branch_list;
        self
    }

    pub fn with_shallow_clone(mut self, shallow_choice: bool) -> Self {
        self.use_shallow = shallow_choice;
        self
    }

    pub fn with_poll_freq(mut self, frequency: Duration) -> Self {
        self.poll_freq = frequency;
        self
    }

    pub fn state(self) -> Option<GitRepoState> {
        self.state
    }

    // Perform shallow clone, update the internal state, and return current `GitRepoState`
    pub async fn update_state(mut self) -> Result<GitRepoState> {
        let temp_path = Temp::new_dir()?;

        match &self.use_shallow {
            true => {
                debug!("Shallow clone");
                self.repo = self.repo.git_clone_shallow(&temp_path.as_path())?;
            }
            false => {
                debug!("Deep clone");
                self.repo = self.repo.git_clone(&temp_path.as_path())?;
            }
        };

        //// DEBUG: list files from temp path
        //let paths = std::fs::read_dir(temp_path.as_path()).unwrap();

        //for path in paths {
        //    debug!("Name: {}", path.unwrap().path().display())
        //}

        // Read the repo and analyze and build report
        //
        let mut repo_report = GitRepoState::default();

        // Collect the branch HEADs
        // If we have a branch filter list, then stick to that list
        let branch_heads = self
            .repo
            .clone()
            .get_remote_branch_head_refs(self.branch_filter.clone())?;

        repo_report.branch_heads = branch_heads;
        repo_report.last_updated = Some(Utc::now());

        // Explicitly delete the clone
        //temp_path.release();
        self.state = Some(repo_report.clone());

        Ok(repo_report)
    }

    // FIXME: Can this be modified so a closure can access state?
    pub fn watch_new_commits<F>(&mut self, closure: F) -> Result<()>
    where
        F: FnOnce() + Copy,
    {
        // We are going to clone the repo into this directory
        let temp_dir = Temp::new_dir().unwrap();

        match &self.use_shallow {
            true => {
                debug!("Shallow clone");
                self.repo = self.repo.git_clone_shallow(&temp_dir.as_path()).unwrap()
            }
            false => {
                debug!("Deep clone");
                self.repo = self.repo.git_clone(&temp_dir.as_path())?
            }
        };

        let mut branch_heads_state = self
            .repo
            .get_remote_branch_head_refs(self.branch_filter.clone())?;

        loop {
            sleep(self.poll_freq);

            let snapshot = self
                .repo
                .get_remote_branch_head_refs(self.branch_filter.clone())?;

            for (branch, commit) in snapshot.clone() {
                match branch_heads_state.get(&branch) {
                    Some(c) => {
                        if &commit == c {
                            info!("No new commits in branch {} found", branch);
                        } else {
                            info!("New commit in branch {} found", branch);
                            closure();
                        }
                    }
                    None => {
                        info!("New branch '{}' found", branch);
                        closure();
                    }
                }
            }

            branch_heads_state = snapshot;
        }
    }
}
