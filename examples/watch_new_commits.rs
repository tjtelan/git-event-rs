use eyre::Result;

use git_event::GitRepoWatchHandler;

#[tokio::main]
async fn main() -> Result<()> { 

    // You probably want to change this to a repo that you own.
    // Push a new commit. Pushing to a new branch work too.
    let test_url = "https://github.com/rust-lang/crates.io-index.git";

    let mut watcher = GitRepoWatchHandler::new(test_url)?.with_shallow_clone(true);

    let _ = watcher.watch_new_commits(move || println!("Hello new commit"));

    Ok(())
}
