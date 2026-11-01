//! Shared fixtures for the graph view tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use herdr_vcs::graph::{Commit, Ref};

mod geometry;
mod lanes;
mod layout;
mod loading;
mod paint;
mod scroll;
mod selection;
mod state;
mod state_loading;
mod tab;

/// A commit with just the fields the view model reads.
fn commit(id: &str, parents: &[&str]) -> Commit {
    commit_with_refs(id, parents, Vec::new())
}

fn commit_with_refs(id: &str, parents: &[&str], refs: Vec<Ref>) -> Commit {
    Commit {
        id: id.to_owned(),
        parents: parents.iter().map(|parent| (*parent).to_owned()).collect(),
        author: "a".to_owned(),
        time: 0,
        refs,
        subject: String::new(),
    }
}
