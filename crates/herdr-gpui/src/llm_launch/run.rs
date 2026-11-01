//! Starting a model server: runs the row's `start` command through `sh`.
//! Blocking; only a background task calls it.

use std::process::Stdio;

/// Why a start command did not succeed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum StartError {
    /// The command ran and exited non-zero; `None` when a signal ended it.
    #[error("start command failed")]
    Failed(Option<i32>),
    /// The shell could not be started.
    #[error("{0}")]
    Spawn(#[source] std::io::Error),
}

/// Runs `command` with `sh -c` and waits for it. Blocking.
pub(crate) fn start(command: &str) -> Result<(), StartError> {
    let mut child = herdr_client::new_command("/bin/sh");
    child
        .args(["-c", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::login_env::apply(&mut child);
    let status = child.status().map_err(StartError::Spawn)?;
    if status.success() {
        Ok(())
    } else {
        Err(StartError::Failed(status.code()))
    }
}
