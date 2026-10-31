//! Reading the commit log of a checkout: one read-only Git child.

use super::{
    error::Error,
    parse::{Commit, MAX_INPUT, parse_log},
};
use std::time::Instant;

/// Most commits one request may ask for.
pub const MAX_LIMIT: u32 = 10_000;

const FORMAT: &str = "--format=%H%x00%P%x00%an%x00%at%x00%D%x00%s%x1e";

/// The newest `limit` commits of every ref in `checkout`, newest first and
/// parents after children. Reads only; it blocks on a child process, so call
/// it from a worker.
pub fn log(
    checkout: &str,
    limit: u32,
    deadline: Instant,
    cancelled: &impl Fn() -> bool,
) -> crate::Result<Vec<Commit>> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(Error::Limit { max: MAX_LIMIT }.into());
    }
    let limit = limit.to_string();
    let output = crate::git::git_bytes(
        checkout,
        &[
            "log",
            "--all",
            "--date-order",
            "--decorate=full",
            "--no-show-signature",
            "-n",
            &limit,
            FORMAT,
            "--end-of-options",
        ],
        "read the commit log",
        deadline,
        cancelled,
        MAX_INPUT,
    )?;
    Ok(parse_log(&output)?)
}
