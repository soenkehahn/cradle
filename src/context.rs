//! An internal module used for testing cradle.

use std::{pin::Pin, task::Poll};
use tokio::io::AsyncWrite;

#[derive(Clone, Debug)]
pub(crate) struct Stdout;

impl AsyncWrite for Stdout {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut tokio::io::stdout()), context, buffer)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut tokio::io::stdout()), context)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut tokio::io::stdout()), context)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Stderr;

impl AsyncWrite for Stderr {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut tokio::io::stderr()), context, buffer)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _context: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _context: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub(crate) struct Context<Stdout, Stderr> {
    pub(crate) stdout: Stdout,
    pub(crate) stderr: Stderr,
}

impl Context<Stdout, Stderr> {
    pub(crate) fn production() -> Self {
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

    #[derive(Clone, Debug)]
    pub(crate) struct TestOutput(Arc<Mutex<Cursor<Vec<u8>>>>);

    impl TestOutput {
        fn new() -> TestOutput {
            TestOutput(Arc::new(Mutex::new(Cursor::new(Vec::new()))))
        }
    }

    impl AsyncWrite for TestOutput {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut std::task::Context<'_>,
            buffer: &[u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            let mut lock = self.0.lock().unwrap();
            Poll::Ready(std::io::Write::write(&mut *lock, buffer))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            todo!()
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _context: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            todo!()
        }
    }

    impl Context<TestOutput, TestOutput> {
        pub(crate) fn test() -> Self {
            Context {
                stdout: TestOutput::new(),
                stderr: TestOutput::new(),
            }
        }

        pub(crate) fn stdout(&self) -> String {
            let lock = self.stdout.0.lock().unwrap();
            String::from_utf8(lock.clone().into_inner()).unwrap()
        }

        pub(crate) fn stderr(&self) -> String {
            let lock = self.stderr.0.lock().unwrap();
            String::from_utf8(lock.clone().into_inner()).unwrap()
        }
    }
}
