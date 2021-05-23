use std::fmt::Display;

#[test]
fn capturing_stdout() {
    use stir::*;

    let output: String = cmd!("echo foo");
    assert_eq!(output, "foo\n");
}

#[test]
#[should_panic(expected = "false:\n  exited with exit code: 1")]
fn panics_on_non_zero_exit_codes() {
    use stir::*;

    cmd_unit!("false");
}

#[test]
fn result_succeeding() {
    use stir::*;

    #[cfg(unix)]
    fn test() -> Result<(), Error> {
        // make sure 'ls' is installed
        let result: Result<(), Error> = cmd!("which ls");
        result?;
        Ok(())
    }

    #[cfg(windows)]
    fn test() -> Result<(), Error> {
        // make sure 'ls' is installed
        let result: Result<(), Error> = cmd!("where ls");
        result?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn result_failing() {
    use stir::*;

    #[cfg(unix)]
    fn test() -> Result<(), Error> {
        let result: Result<(), Error> = cmd!("which does-not-exist");
        result?;
        Ok(())
    }

    #[cfg(windows)]
    fn test() -> Result<(), Error> {
        let result: Result<(), Error> = cmd!("where does-not-exist");
        result?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        "which does-not-exist:\n  exited with exit code: 1"
    );
}

#[test]
fn trimmed_stdout() {
    use std::path::PathBuf;
    use stir::*;

    #[cfg(unix)]
    {
        let ls_path: String = cmd!("which ls");
        let ls_path = ls_path.trim();
        assert!(
            dbg!(PathBuf::from(&ls_path)).exists(),
            "{:?} does not exist",
            &ls_path
        );
    };

    #[cfg(windows)]
    {
        let ls_path: String = cmd!("which ls");
        let ls_path = ls_path.trim();
        assert!(
            dbg!(PathBuf::from(&ls_path)).exists(),
            "{:?} does not exist",
            &ls_path
        );
    };
}

#[test]
fn trimmed_stdout_and_results() {
    use std::path::PathBuf;
    use stir::*;

    #[cfg(unix)]
    fn test() -> Result<(), Error> {
        let result: Result<String, Error> = cmd!("which ls");
        let ls_path = result?;
        let ls_path = ls_path.trim();
        assert!(
            PathBuf::from(&ls_path).exists(),
            "{:?} does not exist",
            &ls_path
        );
        Ok(())
    }

    #[cfg(windows)]
    fn test() -> Result<(), Error> {
        let result: Result<String, Error> = cmd!("where ls");
        let ls_path = result?;
        let ls_path = ls_path.trim();
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
    use stir::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[cfg(unix)]
    fn test() -> MyResult<()> {
        let result: Result<(), Error> = cmd!("which ls");
        result?;
        Ok(())
    }

    #[cfg(windows)]
    fn test() -> MyResult<()> {
        let result: Result<(), Error> = cmd!("where ls");
        result?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn box_dyn_errors_failing() {
    use stir::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[cfg(unix)]
    fn test() -> MyResult<()> {
        let result: Result<(), Error> = cmd!("which does-not-exist");
        result?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        "which does-not-exist:\n  exited with exit code: 1"
    );
}

#[test]
fn user_supplied_errors_succeeding() {
    use stir::*;

    #[derive(Debug)]
    enum Error {
        CmdError(stir::Error),
    }

    impl From<stir::Error> for Error {
        fn from(error: stir::Error) -> Self {
            Error::CmdError(error)
        }
    }

    #[cfg(unix)]
    fn test() -> Result<(), Error> {
        let result: Result<(), stir::Error> = cmd!("which ls");
        result?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn user_supplied_errors_failing() {
    use stir::*;

    #[derive(Debug)]
    enum Error {
        CmdError(stir::Error),
    }

    impl From<stir::Error> for Error {
        fn from(error: stir::Error) -> Self {
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

    #[cfg(unix)]
    fn test() -> Result<(), Error> {
        let result: Result<(), stir::Error> = cmd!("which does-not-exist");
        result?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        "cmd-error: which does-not-exist:\n  exited with exit code: 1"
    );
}
