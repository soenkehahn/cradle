use std::{
    io::{self, Read, Write},
    process::{ChildStderr, ChildStdout},
    thread::{self, JoinHandle},
};

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
    pub(crate) stdout: Option<Stdout>,
    pub(crate) stderr: Stderr,
}

impl Context<Stdout, Stderr> {
    pub fn production() -> Self {
        Context {
            stdout: Some(Stdout),
            stderr: Stderr,
        }
    }
}

pub(crate) struct Waiter {
    stdout: JoinHandle<io::Result<Vec<u8>>>,
    stderr: JoinHandle<io::Result<()>>,
}

impl Waiter {
    pub(crate) fn join(self) -> io::Result<Vec<u8>> {
        self.stderr
            .join()
            .expect("stderr relaying thread panicked")?;
        self.stdout.join().expect("stdout relaying thread panicked")
    }
}

impl<Stdout, Stderr> Context<Stdout, Stderr>
where
    Stdout: Write + Send + Clone + 'static,
    Stderr: Write + Send + Clone + 'static,
{
    pub(crate) fn spawn_standard_stream_relaying(
        &self,
        mut child_stdout: ChildStdout,
        mut child_stderr: ChildStderr,
    ) -> Waiter {
        let mut context = self.clone();
        let stdout_join_handle = thread::spawn(move || {
            let mut collected_stdout = Vec::new();
            let buffer = &mut [0; 256];
            loop {
                let length = child_stdout.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                if let Some(stdout) = &mut context.stdout {
                    stdout.write_all(&buffer[..length])?;
                }
                collected_stdout.extend(&buffer[..length]);
            }
            Ok(collected_stdout)
        });
        let mut context = self.clone();
        let stderr_join_handle = thread::spawn(move || {
            let buffer = &mut [0; 256];
            loop {
                let length = child_stderr.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                context.stderr.write_all(&buffer[..length])?;
            }
            Ok(())
        });
        Waiter {
            stdout: stdout_join_handle,
            stderr: stderr_join_handle,
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
                stdout: Some(TestOutput::new()),
                stderr: TestOutput::new(),
            }
        }

        pub fn stdout(&self) -> String {
            match &self.stdout {
                None => panic!("test context should have stdout"),
                Some(stdout) => {
                    let lock = stdout.0.lock().unwrap();
                    String::from_utf8(lock.clone().into_inner()).unwrap()
                }
            }
        }

        pub fn stderr(&self) -> String {
            let lock = self.stderr.0.lock().unwrap();
            String::from_utf8(lock.clone().into_inner()).unwrap()
        }
    }
}
