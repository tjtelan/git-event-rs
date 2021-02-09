# git-event

[![Crates.io](https://img.shields.io/crates/v/git-event)](https://crates.io/crates/git-event)
[![docs.rs](https://docs.rs/git-event/badge.svg)](https://docs.rs/git-event/)
[![license](https://img.shields.io/github/license/tjtelan/git-event-rs)](LICENSE)
![Github actions build status](https://github.com/tjtelan/git-event-rs/workflows/git-event/badge.svg)

Watch git repos for new commits on per-branch basis. Supply your own code in a closure to be run when new commits found.

Requires `git` to be installed (for shallow clone support - currently not optional)