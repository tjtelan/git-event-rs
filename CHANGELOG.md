# [0.2.0](https://github.com/tjtelan/git-event-rs/compare/v0.1.0...v0.2.0)
- Added `CHANGELOG.md` (You're reading it!)
- Introduced `git-meta` crate
- Introduced many large breaking changes
- Removed `GitCredentials`. Using `git_meta::GitCredentials` 
- `GitRepoWatchHandler` : Swapped `url` field with `repo` using `git_meta::GitRepo`
- `GitRepoWatchHandler` : Added `state` field to store `GitRepoState` internally
- `GitRepoWatchHandler` : Removed the following impls that were moved into `git-meta`
  - `build_git2_remotecallback`
  - `git_clone`
  - `git_shallow_clone`
  - `get_remote_name`
  - `get_remote_branch_head_refs`
- Using `color-eyre` for error handling

# [0.1.0](https://github.com/tjtelan/git-event-rs/compare/v0.0.1...v0.1.0)
- Added private repo example
- Type ergonomic improvements to API and structs

# [0.0.1](https://github.com/tjtelan/git-event-rs/commit/1699676a8f1704006ed0126164c532978bc284a4)
- Added examples
- Introduced `GitRepoWatchHandler`
- Polling for new commits + running closure in `GitRepoWatchHandler::watch_new_commits`
- Private repo support
- Shallow clone support
- Branch filter support