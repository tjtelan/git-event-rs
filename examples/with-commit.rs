use color_eyre::Result;

use git_event::GitRepoWatchHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // You probably want to change this to a repo that you own.
    // Push a new commit. Pushing to a new branch work too.
    //let test_url = "https://github.com/rust-lang/crates.io-index.git";
    let test_url = "https://github.com/tjtelan/git-event-rs.git";
    let branch = "main".to_string();
    let commit_id = "1699676a8f1704006ed0126164c532978bc284a4".to_string();

    let mut watcher = GitRepoWatchHandler::new(test_url)?
        //.with_path(tempdir.to_path_buf())
        .with_branch(Some(branch))
        .with_commit(Some(commit_id))?;

    println!("Watcher: {:?}", watcher.state().unwrap());

    let state = watcher.update_state().await?;

    println!("Watcher after update: {:?}", state);

    let _ = watcher
        .watch_new_commits(true, move |state| {
            println!();
            println!("Last updated: {:?}", state.last_updated);

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
