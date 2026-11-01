//! Shared fixtures for the graph tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

mod changes;
mod layout;
mod parse;
mod run;

/// A commit with just the fields the layout reads.
fn commit(id: &str, parents: &[&str]) -> Commit {
    Commit {
        id: id.to_owned(),
        parents: parents.iter().map(|parent| (*parent).to_owned()).collect(),
        author: "a".to_owned(),
        time: 0,
        refs: Vec::new(),
        subject: String::new(),
    }
}
