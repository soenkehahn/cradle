use std::fmt::Display;

#[cfg(unix)]
const WHICH: &str = "which";
#[cfg(windows)]
const WHICH: &str = "where";

#[test]
fn capturing_stdout() {
    use cradle::*;

    let StdoutTrimmed(output) = cmd!(%"echo foo");
    assert_eq!(output, "foo");
}

#[test]
#[should_panic(expected = "false:\n  exited with exit code: 1")]
fn panics_on_non_zero_exit_codes() {
    use cradle::*;

    cmd_unit!("false");
}

#[test]
fn result_succeeding() {
    use cradle::*;

    fn test() -> Result<(), Error> {
        // make sure 'ls' is installed
        cmd_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn result_failing() {
    use cradle::*;

    fn test() -> Result<(), Error> {
        cmd_result!(WHICH, "does-not-exist")?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        if cfg!(unix) {
            "which does-not-exist:\n  exited with exit code: 1"
        } else {
            "where does-not-exist:\n  exited with exit code: 1"
        }
    );
}

#[test]
fn trimmed_stdout() {
    use cradle::*;
    use std::path::PathBuf;

    {
        let StdoutTrimmed(ls_path) = cmd!(WHICH, "ls");
        assert!(
            PathBuf::from(&ls_path).exists(),
            "{:?} does not exist",
            &ls_path
        );
    };
}

#[test]
fn trimmed_stdout_and_results() {
    use cradle::*;
    use std::path::PathBuf;

    fn test() -> Result<(), Error> {
        let StdoutTrimmed(ls_path) = cmd_result!(WHICH, "ls")?;
        assert!(
            PathBuf::from(&ls_path).exists(),
            "{:?} does not exist",
            &ls_path
        );
        Ok(())
    }

    test().unwrap();
}

#[test]
fn box_dyn_errors_succeeding() {
    use cradle::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn test() -> MyResult<()> {
        cmd_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn box_dyn_errors_failing() {
    use cradle::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn test() -> MyResult<()> {
        cmd_result!(WHICH, "does-not-exist")?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        if cfg!(unix) {
            "which does-not-exist:\n  exited with exit code: 1"
        } else {
            "where does-not-exist:\n  exited with exit code: 1"
        }
    );
}

#[test]
fn user_supplied_errors_succeeding() {
    use cradle::*;

    #[derive(Debug)]
    enum Error {
        CmdError(cradle::Error),
    }

    impl From<cradle::Error> for Error {
        fn from(error: cradle::Error) -> Self {
            Error::CmdError(error)
        }
    }

    fn test() -> Result<(), Error> {
        cmd_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn user_supplied_errors_failing() {
    use cradle::*;

    #[derive(Debug)]
    enum Error {
        CmdError(cradle::Error),
    }

    impl From<cradle::Error> for Error {
        fn from(error: cradle::Error) -> Self {
            Error::CmdError(error)
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::CmdError(error) => write!(f, "cmd-error: {}", error),
            }
        }
    }

    fn test() -> Result<(), Error> {
        cmd_result!(WHICH, "does-not-exist")?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        if cfg!(unix) {
            "cmd-error: which does-not-exist:\n  exited with exit code: 1"
        } else {
            "cmd-error: where does-not-exist:\n  exited with exit code: 1"
        }
    );
}
