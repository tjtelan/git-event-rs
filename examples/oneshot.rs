use color_eyre::Result;

use git_event::GitRepoWatchHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let test_url = "https://github.com/rust-lang/crates.io-index.git";

    let mut watcher = GitRepoWatchHandler::new(test_url)?.with_shallow_clone(true);

    let state = watcher.update_state().await?;

    println!("git state: {:#?}", state);

    Ok(())
}
