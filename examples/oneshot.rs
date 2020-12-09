use eyre::Result;

use git_event::GitRepoWatchHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let test_url = "https://github.com/rust-lang/crates.io-index.git";

    let watcher = GitRepoWatchHandler::new(test_url)?.with_shallow_clone(true);

    let state = watcher.oneshot_report().await?;

    println!("git state: {:?}", state);

    Ok(())
}
