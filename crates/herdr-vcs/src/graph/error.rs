//! Git graph failures: log output that is not understood, and requests
//! outside the supported bounds.

/// The part of a log record that failed to parse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Field {
    Author,
    Id,
    Parent,
    /// The record did not have the expected number of NUL-separated fields.
    Shape,
    Time,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("A commit id must be 4 to 64 lowercase hex characters.")]
    InvalidCommit,
    #[error("The history window must be between 1 and {max} commits.")]
    Limit { max: u32 },
    #[error("Log record {index} is malformed ({field:?}).")]
    Malformed { index: usize, field: Field },
    #[error("The log output is larger than {max} bytes.")]
    TooLarge { max: usize },
    #[error("The log output has more than {max} commits.")]
    TooMany { max: usize },
    #[error("The log output ended inside a record.")]
    Truncated,
    // keep-sorted end
}
