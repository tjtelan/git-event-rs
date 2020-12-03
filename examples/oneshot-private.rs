use eyre::Result;

use git_event::GitRepoWatchHandler;

use git_url_parse::GitUrl;
use std::env;

use std::fs::File;
use std::io::prelude::*;

use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();

    // Set an env var for private git repo
    let test_url = env::var("TEST_GIT_URL").expect("This test needs env var TEST_GIT_URL set");
    let test_url = GitUrl::parse(&test_url).unwrap();
    println!("test_url: {:?}", test_url.to_string());

    // Set an env var for the username used for cloning
    let ssh_user = env::var("TEST_SSH_USER").expect("This test needs TEST_SSH_USER set");
    println!("ssh_user: {:?}", ssh_user);

    // Set an env var for location to private key
    let ssh_private_key_path =
        env::var("TEST_SSH_KEY").expect("This test needs TEST_SSH_KEY set to a file path");
    println!("ssh_private_key_path: {:?}", ssh_private_key_path);

    //let mut file = File::open(ssh_private_key_path)?;
    //let mut ssh_private_key = String::new();
    //file.read_to_string(&mut ssh_private_key)?;

    //let ssh_key_passphrase = env::var("TEST_SSH_PASSPHRASE").unwrap_or_default();

    let clone_creds = git_event::GitCredentials::SshKey {
        username: ssh_user,
        public_key: None,
        private_key: ssh_private_key_path,
        passphrase: None,
    };

    println!("clone_creds: {:?}", clone_creds);

    //println!("URI: {}", test_url.trim_auth().to_string());

    let watcher = GitRepoWatchHandler::new(test_url.to_string())?
        .with_credentials(clone_creds)
        .with_shallow_clone(true);

    let state = watcher.oneshot_report().await?;

    println!("git state: {:?}", state);

    Ok(())
}
