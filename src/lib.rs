//use calloop::{generic::Generic, EventLoop, Interest, Mode};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{eyre, Result};
use mktemp::Temp;
use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, path::PathBuf};

use tracing::{debug, error, info};

use git_meta::{GitCommitMeta, GitCredentials, GitRepo};

type BranchHeads = HashMap<String, GitCommitMeta>;
type PathAlert = HashMap<String, Vec<PathBuf>>;

#[derive(Clone, Debug)]
pub struct GitRepoWatchHandler {
    pub repo: GitRepo,
    pub state: Option<GitRepoState>,
    pub branch_filter: Option<Vec<String>>,
    pub path_filter: Option<Vec<PathBuf>>,
    pub use_shallow: bool,
    pub poll_freq: Duration,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitRepoState {
    pub last_updated: Option<DateTime<Utc>>,
    pub branch_heads: BranchHeads,
    pub path_alert: PathAlert,
}

// Create an iterator for BranchHeads

// I need to be able to start the state at an arbitrary place

impl GitRepoWatchHandler {
    pub fn new<U: AsRef<str>>(url: U) -> Result<Self> {
        Ok(GitRepoWatchHandler {
            repo: GitRepo::new(url)?,
            state: None,
            branch_filter: None,
            path_filter: None,
            use_shallow: false,
            poll_freq: Duration::from_secs(5),
        })
    }

    ///// Set the current filesystem path
    //pub fn with_path(mut self, path: PathBuf) -> Self {
    //    self.repo = self.repo.with_path(path);
    //    self
    //}

    /// Set the current repo branch
    pub fn with_branch(mut self, branch: Option<String>) -> Self {
        self.repo = self.repo.with_branch(branch);
        self
    }

    /// Set the current repo commit id.
    /// If you're using `with_commit()` to build a `GitRepoWatcher` with `new()
    /// then use `with_commit()` as the end of the chain
    pub fn with_commit(mut self, id: Option<String>) -> Result<Self> {
        // We're going to do a deep clone in order to build this...
        let tempdir = Temp::new_dir()?;

        // Clone repo and then change to the specific branch/commit, if specified
        let _clone = self.repo.to_clone().git_clone(tempdir.to_path_buf())?;
        let repo = GitRepo::open(tempdir.to_path_buf(), self.repo.branch.clone(), id.clone())?;

        let repo_head = self
            .repo
            .head
            .clone()
            .ok_or_else(|| eyre!("Repo HEAD commit is not set"))?;
        let repo_branch = self
            .repo
            .branch
            .clone()
            .ok_or_else(|| eyre!("Repo branch is not set"))?;

        //println!("opened: {:?}", repo.list_files_changed_at(id.clone().unwrap()));

        let mut path_alert: HashMap<String, Vec<PathBuf>> = HashMap::new();
        // Get the files changed in the HEAD commit
        let changed_paths = repo.to_info().list_files_changed_at(repo_head.id.clone())?;

        //println!("{:?}", &changed_paths);

        // Save list of paths where changes were made
        if let Some(paths) = changed_paths {
            path_alert.insert(repo_branch.clone(), paths);
        }

        // Save opened repo
        self.repo = repo;

        let mut head = HashMap::new();
        head.insert(
            repo_branch,
            GitCommitMeta {
                id: repo_head.id.clone(),
                message: repo_head.message,
                timestamp: repo_head.timestamp,
            },
        );

        let repo_report = GitRepoState {
            branch_heads: head,
            last_updated: Some(Utc::now()),
            path_alert,
        };

        self.state = Some(repo_report);
        self.repo = self.repo.with_commit(id);
        Ok(self)
    }

    pub fn with_credentials(mut self, creds: Option<GitCredentials>) -> Self {
        self.repo.credentials = creds;
        self
    }

    pub fn with_branch_filter(mut self, branch_list: Option<Vec<String>>) -> Self {
        self.branch_filter = branch_list;
        self
    }

    pub fn with_path_filter(mut self, path_list: Option<Vec<PathBuf>>) -> Self {
        self.path_filter = path_list;
        self
    }

    #[cfg(feature = "shallow_clone")]
    pub fn with_shallow_clone(mut self, shallow_choice: bool) -> Self {
        self.use_shallow = shallow_choice;
        self
    }

    pub fn with_poll_freq(mut self, frequency: Duration) -> Self {
        self.poll_freq = frequency;
        self
    }

    ///// Reset the repo and state with the current branch and commit id
    //pub fn reset(mut self) {
    //    // Re-open repo
    //    // Re-init GitCommitMeta
    //}

    pub fn state(&self) -> Option<GitRepoState> {
        self.state.clone()
    }

    fn _update_state(&mut self) -> Result<GitRepoState> {
        let prev_state = self.clone();

        // Re-clone the repo
        let temp_path = Temp::new_dir()?;

        self.repo = if cfg!(feature = "shallow_clone") {
            match &self.use_shallow {
                true => {
                    debug!("Shallow clone");
                    self.repo.to_clone().git_clone_shallow(&temp_path.as_path())?
                }
                false => {
                    debug!("Deep clone");
                    self.repo.to_clone().git_clone(&temp_path.as_path())?
                }
            }
        } else {
            debug!("Deep clone");
            self.repo.to_clone().git_clone(&temp_path.as_path())?
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
            .to_info()
            .get_remote_branch_head_refs(self.branch_filter.clone())?;

        repo_report.branch_heads = branch_heads.clone();

        // Update any active path triggers from the previous branch heads
        let mut path_alert = HashMap::new();

        // If there are no existing path filters, then just list all the changed files between commits
        for (branch, commit) in branch_heads {
            // Try to get a previous commit
            if let Some(c) = prev_state
                .state()
                .unwrap_or_default()
                .branch_heads
                .get(&branch)
            {
                let paths = self
                    .repo
                    .to_info()
                    .list_files_changed_between(commit.id, c.clone().id)?;
                if let Some(p) = paths {
                    path_alert.insert(branch, p);
                } else {
                    error!("There are no ")
                }
                //else {
                //    println!("DEBUG: No changes in branch {} found", &branch);
                //}
            } else {
                let paths = self.repo.to_info().list_files_changed_at(commit.id)?;
                if let Some(p) = paths {
                    path_alert.insert(branch, p);
                }
            };
        }

        repo_report.path_alert = path_alert;
        repo_report.last_updated = Some(Utc::now());

        // Explicitly delete the clone
        //temp_path.release();
        self.state = Some(repo_report.clone());

        Ok(repo_report)
    }

    // Perform shallow clone, update the internal state, and return current `GitRepoState`
    pub async fn update_state(&mut self) -> Result<GitRepoState> {
        self._update_state()
    }

    // Sync version of `update_state()`
    pub fn update_state_sync(&mut self) -> Result<GitRepoState> {
        self._update_state()
    }

    pub async fn watch_new_commits(
        &mut self,
        pre_run: bool,
        closure: impl Fn(GitRepoState),
    ) -> Result<()> {
        let mut branch_heads_state = self.update_state().await?;

        if pre_run {
            closure(branch_heads_state.clone());
        }

        // New commit check
        loop {
            sleep(self.poll_freq);

            let snapshot = self.get_branches_snapshot()?;
            branch_heads_state = self.update_state().await?;

            // Loop over all of the branches and the last commits we saw them at
            self.run_code_if_new_commit_in_branch(branch_heads_state, snapshot.clone(), &closure)?;
        }
    }

    fn run_code_if_new_commit_in_branch(
        &self,
        current_state: GitRepoState,
        current_commits: HashMap<String, GitCommitMeta>,
        closure: impl Fn(GitRepoState) + Copy,
    ) -> Result<bool> {
        for (branch, commit) in current_commits {
            match current_state.branch_heads.get(&branch) {
                Some(c) => {
                    if &commit == c {
                        info!("No new commits in branch {} found", branch);
                    } else {
                        info!("New commit in branch {} found", branch);

                        if let Some(state) = self.state() {
                            closure(state);
                        } else {
                            return Err(eyre!("No state found"));
                        }
                    }
                }
                None => {
                    info!("New branch '{}' found", branch);
                    if let Some(state) = self.state() {
                        closure(state);
                    } else {
                        return Err(eyre!("No state found"));
                    }
                }
            }
        }
        Ok(true)
    }
    pub fn watch_new_commits_sync(
        &mut self,
        pre_run: bool,
        closure: impl Fn(GitRepoState),
    ) -> Result<()> {
        if pre_run {
            if let Some(state) = self.state() {
                closure(state);
            } else {
                return Err(eyre!("No state found"));
            }
        }

        loop {
            sleep(self.poll_freq);

            let snapshot = self.get_branches_snapshot()?;
            let branch_heads_state = self.update_state_sync()?;

            // Loop over all of the branches and the last commits we saw them at
            self.run_code_if_new_commit_in_branch(branch_heads_state, snapshot.clone(), &closure)?;
        }
    }

    fn get_branches_snapshot(&self) -> Result<HashMap<String, GitCommitMeta>> {
        if let Some(state) = self.state.clone() {
            Ok(state.branch_heads)
        } else {
            Err(eyre!("Unable to get snapshot of branch HEAD refs"))
        }
    }
}
