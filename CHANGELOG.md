# [0.4.1](https://github.com/tjtelan/git-event-rs/compare/v0.4.0...v0.4.1) (2022-01-22)
- Minor fix - Update dependencies
# [0.4.0](https://github.com/tjtelan/git-event-rs/compare/v0.3.0...v0.4.0) (2021-12-20)
- Migrate to Rust 2021
- Change panic behavior to return `Err()`
- Replace `log` crate with `tracing`
- Add feature `shallow_clone` to allow opt-out of using CLI `git` installed in PATH.
# [0.3.0](https://github.com/tjtelan/git-event-rs/compare/v0.2.1...v0.3.0) (2021-02-27)
- Updating `watch_new_commits` to be `async`
- Updating `watch_new_commits` input closure, to return `GitRepoState` to user
- Updating `watch_new_commits` params with `pre_run` to call the closure once before polling for updates
- Offering a sync version of `watch_new_commits` called `watch_new_commits_sync`
- Changed `update_state` input to `&mut self`
- `update_state` sets changed paths between commits.
# [0.2.1](https://github.com/tjtelan/git-event-rs/compare/v0.2.0...v0.2.1) (2021-02-12)
- Updating `git-meta` to `^0.2`
# [0.2.0](https://github.com/tjtelan/git-event-rs/compare/v0.1.0...v0.2.0) (2021-02-08)
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

# [0.1.0](https://github.com/tjtelan/git-event-rs/compare/v0.0.1...v0.1.0) (2020-12-09)
- Added private repo example
- Type ergonomic improvements to API and structs

# [0.0.1](https://github.com/tjtelan/git-event-rs/commit/1699676a8f1704006ed0126164c532978bc284a4) (2020-11-01)
- Added examples
- Introduced `GitRepoWatchHandler`
- Polling for new commits + running closure in `GitRepoWatchHandler::watch_new_commits`
- Private repo support
- Shallow clone support
- Branch filter support