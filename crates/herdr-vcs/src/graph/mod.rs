//! The Git graph: the commit log of a checkout and the lanes that draw it.

mod error;
mod layout;
mod parse;
mod run;

pub use error::{Error, Field};
pub use layout::{Edge, EdgeKind, Layout, Row, layout};
pub use parse::{Commit, MAX_COMMITS, MAX_INPUT, Ref, parse_log};
pub use run::{MAX_LIMIT, log};

#[cfg(test)]
mod tests;
