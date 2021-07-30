//! An internal module used for testing cradle.

use std::io::{self, Write};

#[derive(Clone)]
pub struct Stdout;

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

#[derive(Clone)]
pub struct Stderr;

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stderr().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stderr().flush()
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct Context<Stdout, Stderr> {
    pub(crate) stdout: Stdout,
    pub(crate) stderr: Stderr,
}

impl Context<Stdout, Stderr> {
    pub fn production() -> Self {
        Context {
            stdout: Stdout,
            stderr: Stderr,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{
        io::Cursor,
        sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    pub(crate) struct TestOutput(Arc<Mutex<Cursor<Vec<u8>>>>);

    impl TestOutput {
        fn new() -> TestOutput {
            TestOutput(Arc::new(Mutex::new(Cursor::new(Vec::new()))))
        }
    }

    impl Write for TestOutput {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut lock = self.0.lock().unwrap();
            lock.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            let mut lock = self.0.lock().unwrap();
            lock.flush()
        }
    }

    impl Context<TestOutput, TestOutput> {
        pub(crate) fn test() -> Self {
            Context {
                stdout: TestOutput::new(),
                stderr: TestOutput::new(),
            }
        }

        pub fn stdout(&self) -> String {
            let lock = self.stdout.0.lock().unwrap();
            String::from_utf8(lock.clone().into_inner()).unwrap()
        }

        pub fn stderr(&self) -> String {
            let lock = self.stderr.0.lock().unwrap();
            String::from_utf8(lock.clone().into_inner()).unwrap()
        }
    }
}
