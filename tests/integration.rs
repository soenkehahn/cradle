#[cfg(unix)]
const WHICH: &str = "which";
#[cfg(windows)]
const WHICH: &str = "where";

#[test]
fn runs_child_processes() {
    use cradle::prelude::*;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    run!(CurrentDir(temp_dir.path()), %"touch foo");
    assert!(temp_dir.path().join("foo").is_file());
}

#[test]
#[should_panic(expected = "false:\n  exited with exit code: 1")]
fn panics_on_non_zero_exit_codes() {
    use cradle::prelude::*;

    run!("false");
}

#[test]
fn capturing_stdout() {
    use cradle::prelude::*;

    let StdoutTrimmed(output) = run_output!(%"echo foo");
    assert_eq!(output, "foo");
}

#[test]
fn result_succeeding() {
    use cradle::prelude::*;

    fn test() -> Result<(), Error> {
        // make sure 'ls' is installed
        run_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn result_failing() {
    use cradle::prelude::*;

    fn test() -> Result<(), Error> {
        run_result!(WHICH, "does-not-exist")?;
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
    use cradle::prelude::*;
    use std::path::PathBuf;

    {
        let StdoutTrimmed(ls_path) = run_output!(WHICH, "ls");
        assert!(
            PathBuf::from(&ls_path).exists(),
            "{:?} does not exist",
            &ls_path
        );
    };
}

#[test]
fn trimmed_stdout_and_results() {
    use cradle::prelude::*;
    use std::path::PathBuf;

    fn test() -> Result<(), Error> {
        let StdoutTrimmed(ls_path) = run_result!(WHICH, "ls")?;
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
    use cradle::prelude::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn test() -> MyResult<()> {
        run_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn box_dyn_errors_failing() {
    use cradle::prelude::*;

    type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn test() -> MyResult<()> {
        run_result!(WHICH, "does-not-exist")?;
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
    use cradle::prelude::*;

    #[derive(Debug)]
    enum Error {
        Cradle(cradle::Error),
    }

    impl From<cradle::Error> for Error {
        fn from(error: cradle::Error) -> Self {
            Error::Cradle(error)
        }
    }

    fn test() -> Result<(), Error> {
        run_result!(WHICH, "ls")?;
        Ok(())
    }

    test().unwrap();
}

#[test]
fn user_supplied_errors_failing() {
    use cradle::prelude::*;
    use std::fmt::Display;

    #[derive(Debug)]
    enum Error {
        Cradle(cradle::Error),
    }

    impl From<cradle::Error> for Error {
        fn from(error: cradle::Error) -> Self {
            Error::Cradle(error)
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::Cradle(error) => write!(f, "cradle error: {}", error),
            }
        }
    }

    fn test() -> Result<(), Error> {
        run_result!(WHICH, "does-not-exist")?;
        Ok(())
    }

    assert_eq!(
        test().unwrap_err().to_string(),
        if cfg!(unix) {
            "cradle error: which does-not-exist:\n  exited with exit code: 1"
        } else {
            "cradle error: where does-not-exist:\n  exited with exit code: 1"
        }
    );
}

mod run_interface {
    use super::*;

    #[test]
    fn result_succeeding() {
        use cradle::prelude::*;

        fn test() -> Result<(), Error> {
            // make sure 'ls' is installed
            (WHICH, "ls").run_result()?;
            Ok(())
        }

        test().unwrap();
    }

    #[test]
    fn result_failing() {
        use cradle::prelude::*;

        fn test() -> Result<(), Error> {
            (WHICH, "does-not-exist").run_result()?;
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
    fn box_dyn_errors_succeeding() {
        use cradle::prelude::*;

        type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

        fn test() -> MyResult<()> {
            (WHICH, "ls").run_result()?;
            Ok(())
        }

        test().unwrap();
    }

    #[test]
    fn box_dyn_errors_failing() {
        use cradle::prelude::*;

        type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

        fn test() -> MyResult<()> {
            (WHICH, "does-not-exist").run_result()?;
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
        use cradle::prelude::*;

        #[derive(Debug)]
        enum Error {
            Cradle(cradle::Error),
        }

        impl From<cradle::Error> for Error {
            fn from(error: cradle::Error) -> Self {
                Error::Cradle(error)
            }
        }

        fn test() -> Result<(), Error> {
            (WHICH, "ls").run_result()?;
            Ok(())
        }

        test().unwrap();
    }

    #[test]
    fn user_supplied_errors_failing() {
        use cradle::prelude::*;
        use std::fmt::Display;

        #[derive(Debug)]
        enum Error {
            Cradle(cradle::Error),
        }

        impl From<cradle::Error> for Error {
            fn from(error: cradle::Error) -> Self {
                Error::Cradle(error)
            }
        }

        impl Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Error::Cradle(error) => write!(f, "cradle error: {}", error),
                }
            }
        }

        fn test() -> Result<(), Error> {
            (WHICH, "does-not-exist").run_result()?;
            Ok(())
        }

        assert_eq!(
            test().unwrap_err().to_string(),
            if cfg!(unix) {
                "cradle error: which does-not-exist:\n  exited with exit code: 1"
            } else {
                "cradle error: where does-not-exist:\n  exited with exit code: 1"
            }
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn memory_test() {
    use cradle::prelude::*;
    run!(%"cargo build -p memory-tests --release");
    run!(%"cargo run -p memory-tests --bin run");
}
