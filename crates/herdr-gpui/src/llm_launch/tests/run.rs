use super::*;

#[test]
fn exit_zero_is_ok() {
    assert!(start("exit 0").is_ok());
}

#[test]
fn a_non_zero_exit_reports_its_code() {
    assert!(matches!(start("exit 3"), Err(StartError::Failed(Some(3)))));
}

#[test]
fn a_command_that_does_not_exist_fails() {
    // The shell answers 127; a shell that cannot start is a spawn error.
    assert!(matches!(
        start("/nonexistent/x"),
        Err(StartError::Failed(Some(127)) | StartError::Spawn(_))
    ));
}

#[test]
fn shell_syntax_works() {
    assert!(start("true && true").is_ok());
    assert!(matches!(
        start("true && exit 4"),
        Err(StartError::Failed(Some(4)))
    ));
}

#[test]
fn the_environment_is_expanded() {
    assert!(start("test -n \"$HOME\"").is_ok());
}

#[test]
fn a_signal_reports_no_code() {
    assert!(matches!(start("kill -9 $$"), Err(StartError::Failed(None))));
}
