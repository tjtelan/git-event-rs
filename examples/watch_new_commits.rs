use color_eyre::Result;

use git_event::GitRepoWatchHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // You probably want to change this to a repo that you own.
    // Push a new commit. Pushing to a new branch work too.
    let test_url = "https://github.com/rust-lang/crates.io-index.git";

    let mut watcher = GitRepoWatchHandler::new(test_url)?.with_shallow_clone(true);

    let _ = watcher
        .watch_new_commits(true, move |state| {
            println!();
            println!("Last updated: {:#?}", state.last_updated);

            for (branch, meta) in state.branch_heads {
                println!("Branch: {}", branch);
                println!("Commit id: {}", meta.id);
                println!("Commit message: {}", meta.message.unwrap());
                println!("Timestamp: {:?}", meta.timestamp.unwrap());
                println!();
            }
        })
        .await;

    Ok(())
}
