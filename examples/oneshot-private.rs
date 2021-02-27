use color_eyre::Result;

use git_event::GitRepoWatchHandler;

use git_meta;
use git_url_parse::GitUrl;
use std::env;
use std::path::PathBuf;

use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "oneshot_private");

    let _ = env_logger::try_init();

    // Set an env var for private git repo
    let test_url = env::var("TEST_GIT_URL").expect("This test needs env var TEST_GIT_URL set");
    let test_url = GitUrl::parse(&test_url).unwrap();
    info!("TEST_GIT_URL: {:?}", test_url);

    // Set an env var for the username used for cloning
    let ssh_user = env::var("TEST_SSH_USER").expect("This test needs TEST_SSH_USER set");
    info!("TEST_SSH_USER: {:?}", ssh_user);

    // Set an env var for location to private key
    let ssh_private_key_path = PathBuf::from(
        env::var("TEST_SSH_KEY").expect("This test needs TEST_SSH_KEY set to a file path"),
    );
    info!("TEST_SSH_KEY: {:?}", ssh_private_key_path);

    let clone_creds = git_meta::GitCredentials::SshKey {
        username: ssh_user,
        public_key: None,
        private_key: ssh_private_key_path,
        passphrase: None,
    };

    let mut watcher = GitRepoWatchHandler::new(test_url.to_string())?
        .with_credentials(Some(clone_creds))
        .with_shallow_clone(true);

    let state = &watcher.update_state().await?;

    println!("git state: {:?}", state);

    Ok(())
}
