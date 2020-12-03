//use calloop::{generic::Generic, EventLoop, Interest, Mode};
use eyre::Result;
use git2::Cred;
use git_url_parse::GitUrl;
use mktemp::Temp;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::thread::sleep;
use std::time::Duration;

use log::{debug, info};

#[derive(Debug, Clone)]
pub enum GitCredentials {
    SshKey {
        username: String,
        public_key: Option<String>,
        private_key: String,
        passphrase: Option<String>,
    },
    UserPassPlaintext {
        username: String,
        password: String,
    },
}

impl GitCredentials {
    //pub fn ssh_key<P: AsRef<Path>>(username: String, publickey: Option<P>, privatekey: P, passphrase: Option<String>) -> Self{

    //    let pk = if let Some(pk) = publickey {
    //        Some(pk.as_ref())
    //    } else { None };
    //    let pp = if let Some(pp) = passphrase {
    //        Some(pp.as_str())
    //    } else { None };

    //    GitCredentials::SshKey {
    //        username: username.as_str(),
    //        publickey: pk,
    //        privatekey: privatekey.as_ref(),
    //        passphrase: pp,
    //    }
    //}
}

type BranchHeads = HashMap<String, GitCommitMeta>;

#[derive(Clone, Debug)]
pub struct GitRepoWatchHandler {
    pub url: GitUrl,
    pub credentials: Option<GitCredentials>,
    pub branch_filter: Option<Vec<String>>,
    pub use_shallow: bool,
    //branch_heads: Option<BranchHeads>,
    // TODO:
    //path_filter: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitRepoState {
    pub url: GitUrl,
    pub branch_heads: BranchHeads,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GitCommitMeta {
    pub id: Vec<u8>,
    pub message: Option<String>,
    pub epoch_time: i64,
}

// trait bound AsRef<Path> for GitCredentials?
impl GitRepoWatchHandler {
    pub fn new<U: AsRef<str>>(url: U) -> Result<Self> {
        Ok(GitRepoWatchHandler {
            url: GitUrl::parse(url.as_ref()).expect("Provided git url failed parsing"),
            credentials: None,
            branch_filter: None,
            use_shallow: false,
            //branch_heads: None,
        })
    }

    pub fn with_credentials(mut self, creds: GitCredentials) -> Self {
        self.credentials = Some(creds);
        self
    }

    pub fn with_branch_filter(mut self, branch_list: Vec<String>) -> Self {
        self.branch_filter = Some(branch_list);
        self
    }

    pub fn with_shallow_clone(mut self, shallow_choice: bool) -> Self {
        self.use_shallow = shallow_choice;
        self
    }

    //pub fn update_branch_heads(mut self, branch_heads: Option<BranchHeads>) -> Self {
    //    self.branch_heads = branch_heads;
    //    self
    //}

    // Perform shallow clone then return current GitRepoState
    pub async fn oneshot_report(self) -> Result<GitRepoState> {
        let temp_path = Temp::new_dir()?;

        let repo_ref = match &self.use_shallow {
            true => {
                println!("Shallow clone");
                self.shallow_git_clone(&temp_path.as_path())?
            }
            false => {
                println!("Deep clone");
                self.git_clone(&temp_path.as_path())?
            }
        };

        // DEBUG: list files from temp path
        let paths = std::fs::read_dir(temp_path.as_path()).unwrap();

        for path in paths {
            debug!("Name: {}", path.unwrap().path().display())
        }

        // Read the repo and analyze and build report
        //
        let mut repo_report = GitRepoState::default();

        repo_report.url = self.url.clone();

        // Collect the branch HEADs
        // If we have a branch filter list, then stick to that list
        let branch_heads =
            self.get_remote_branch_head_refs(repo_ref, self.branch_filter.clone())?;
        repo_report.branch_heads = branch_heads;

        // Explicitly delete the clone
        //temp_path.release();

        Ok(repo_report)
    }

    pub fn watch_new_commits<F>(&mut self, closure: F) -> Result<()>
    where
        F: FnOnce() + Copy,
    {
        // Initial state
        let temp_path = Temp::new_dir().unwrap();

        let repo_ref = match &self.use_shallow {
            true => {
                println!("Shallow clone");
                self.shallow_git_clone(&temp_path.as_path()).unwrap()
            }
            false => {
                println!("Deep clone");
                self.git_clone(&temp_path.as_path()).unwrap()
            }
        };

        let mut branch_heads_state =
            self.get_remote_branch_head_refs(repo_ref, self.branch_filter.clone())?;

        loop {
            sleep(Duration::from_secs(5));

            let repo = git2::Repository::open(&temp_path)?;

            let snapshot = self.get_remote_branch_head_refs(repo, self.branch_filter.clone())?;
            //self.get_remote_branch_head_refs(repo, self.branch_filter.clone()).unwrap_or(panic!("Failed to get branch heads"));

            for (branch, commit) in snapshot.clone() {
                match branch_heads_state.get(&branch) {
                    Some(c) => {
                        if &commit == c {
                            println!("No new commits in branch {} found", branch);
                        } else {
                            println!("New commit in branch {} found", branch);
                            closure();
                        }
                    }
                    None => {
                        println!("New branch '{}' found", branch);
                        closure();
                    }
                }
            }

            branch_heads_state = snapshot;
        }
    }

    fn build_git2_remotecallback(&self) -> git2::RemoteCallbacks {
        // Configure creds based on auth type, or lack of
        //let mut cb = git2::RemoteCallbacks::new();

        if let Some(cred) = self.credentials.clone() {
            debug!("Before building callback: {:?}", &cred);

            match cred {
                GitCredentials::SshKey {
                    username,
                    public_key,
                    private_key,
                    passphrase,
                } => {
                    let mut cb = git2::RemoteCallbacks::new();
                    let privkey_path = std::path::PathBuf::from(private_key);

                    cb.credentials(move |_, _, _| {
                        match (public_key.clone(), passphrase.clone()) {
                            (None, None) => {
                                Ok(Cred::ssh_key(&username, None, privkey_path.as_path(), None)
                                    .expect("Could not create credentials object for ssh key"))
                            }
                            (None, Some(pp)) => Ok(Cred::ssh_key(
                                &username,
                                None,
                                privkey_path.as_path(),
                                Some(pp.as_ref()),
                            )
                            .expect("Could not create credentials object for ssh key")),
                            (Some(pk), None) => {
                                let pubkey_path = std::path::PathBuf::from(pk);

                                Ok(Cred::ssh_key(
                                    &username,
                                    Some(pubkey_path.as_path()),
                                    privkey_path.as_path(),
                                    None,
                                )
                                .expect("Could not create credentials object for ssh key"))
                            }
                            (Some(pk), Some(pp)) => {
                                let pubkey_path = std::path::PathBuf::from(pk);

                                Ok(Cred::ssh_key(
                                    &username,
                                    Some(pubkey_path.as_path()),
                                    privkey_path.as_path(),
                                    Some(pp.as_ref()),
                                )
                                .expect("Could not create credentials object for ssh key"))
                            }
                        }
                    });

                    cb
                }
                GitCredentials::UserPassPlaintext { username, password } => {
                    let mut cb = git2::RemoteCallbacks::new();
                    cb.credentials(move |_, _, _| {
                        Cred::userpass_plaintext(username.as_str(), password.as_str())
                    });

                    cb
                }
            }
        } else {
            // No credentials. Repo is public
            git2::RemoteCallbacks::new()
        }

        //cb
    }

    fn git_clone<P: AsRef<Path>>(&self, target: P) -> Result<git2::Repository> {
        // Configure creds
        //let cb = self.build_git2_remotecallback();

        let mut cb = git2::RemoteCallbacks::new();
        &cb.credentials(move |_, _, _| {
            Ok(
                Cred::ssh_key("git", None, Path::new("/tmp/deletemesshkey"), None)
                    .expect("Could not create credentials object for ssh key"),
            )
        });

        let mut builder = git2::build::RepoBuilder::new();
        let mut fetch_options = git2::FetchOptions::new();

        fetch_options.remote_callbacks(cb);
        builder.fetch_options(fetch_options);

        let repo = match builder.clone(&self.url.to_string(), target.as_ref()) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        Ok(repo)
    }

    fn shallow_git_clone<P: AsRef<Path>>(&self, target: P) -> Result<git2::Repository> {
        let repo = if let Some(cred) = self.credentials.clone() {
            match cred {
                GitCredentials::SshKey {
                    username,
                    public_key: _,
                    private_key,
                    passphrase: _,
                } => {
                    let mut parsed_uri = self.url.trim_auth();
                    parsed_uri.user = Some(username.to_string());

                    let shell_clone_command = Command::new("git")
                        .arg("clone")
                        .arg(format!("{}", parsed_uri))
                        .arg(format!("{}", target.as_ref().display()))
                        .arg("--no-single-branch")
                        .arg("--depth=1")
                        .arg("--config")
                        .arg(format!(
                            "core.sshcommand=ssh -i {privkey_path}",
                            privkey_path = private_key
                        ))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("failed to run git clone");

                    let clone_out = shell_clone_command
                        .wait_with_output()
                        .expect("failed to open stdout");

                    debug!("Clone output: {:?}", clone_out);

                    git2::Repository::open(target.as_ref())
                        .expect("Failed to open shallow clone dir")
                }
                GitCredentials::UserPassPlaintext { username, password } => {
                    let mut cli_remote_url = self.url.clone();
                    cli_remote_url.user = Some(username.to_string());
                    cli_remote_url.token = Some(password.to_string());

                    let shell_clone_command = Command::new("git")
                        .arg("clone")
                        .arg(format!("{}", cli_remote_url))
                        .arg(format!("{}", target.as_ref().display()))
                        .arg("--no-single-branch")
                        .arg("--depth=1")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("Failed to run git clone");

                    let _clone_out = shell_clone_command.stdout.expect("Failed to open stdout");
                    git2::Repository::open(target.as_ref())
                        .expect("Failed to open shallow clone dir")
                }
            }
        } else {
            let parsed_uri = self.url.trim_auth();

            info!("Url: {}", format!("{}", parsed_uri));
            info!("Directory: {}", format!("{}", target.as_ref().display()));

            let shell_clone_command = Command::new("git")
                .arg("clone")
                .arg(format!("{}", parsed_uri))
                .arg(format!("{}", target.as_ref().display()))
                .arg("--no-single-branch")
                .arg("--depth=1")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to run git clone");

            let _clone_out = shell_clone_command
                .wait_with_output()
                .expect("Failed to wait for output")
                .stdout;

            git2::Repository::open(target.as_ref()).expect("Failed to open shallow clone dir")
        };

        Ok(repo)
    }

    /// Return the remote name from the given Repository
    fn get_remote_name(&self, r: &git2::Repository) -> Result<String> {
        let remote_name = r
            .branch_upstream_remote(
                r.head()
                    .and_then(|h| h.resolve())?
                    .name()
                    .expect("branch name is valid utf8"),
            )
            .map(|b| b.as_str().expect("valid utf8").to_string())
            .unwrap_or_else(|_| "origin".into());

        Ok(remote_name)
    }

    fn get_remote_branch_head_refs(
        &self,
        repo: git2::Repository,
        branch_filter: Option<Vec<String>>,
    ) -> Result<HashMap<String, GitCommitMeta>> {
        let cb = self.build_git2_remotecallback();

        let remote = self
            .get_remote_name(&repo)
            .expect("Could not read remote name from git2::Repository");

        let mut remote = repo
            .find_remote(&remote)
            .or_else(|_| repo.remote_anonymous(&remote))
            .unwrap();

        // Connect to the remote and call the printing function for each of the
        // remote references.
        let connection = remote
            .connect_auth(git2::Direction::Fetch, Some(cb), None)
            .unwrap();

        let git_branch_ref_prefix = "refs/heads/";
        let mut ref_map: HashMap<String, GitCommitMeta> = HashMap::new();

        for git_ref in connection
            .list()?
            .iter()
            .filter(|head| head.name().starts_with(git_branch_ref_prefix))
        {
            let branch_name = git_ref
                .name()
                .to_string()
                .rsplit(git_branch_ref_prefix)
                .collect::<Vec<&str>>()[0]
                .to_string();

            if let Some(ref branches) = branch_filter {
                if branches.contains(&branch_name.to_string()) {
                    continue;
                }
            }

            // Get the commit object
            let commit = repo.find_commit(git_ref.oid())?;

            let head_commit = GitCommitMeta {
                id: commit.id().as_bytes().to_owned(),
                message: commit.message().map_or(None, |m| Some(m.to_string())),
                epoch_time: commit.time().seconds().to_owned(),
            };

            ref_map.insert(branch_name, head_commit);
        }

        Ok(ref_map)
    }
}
