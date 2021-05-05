use std::{
    io::{self, Read, Write},
    process::ChildStdout,
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

#[doc(hidden)]
#[derive(Clone)]
pub struct Context<Stdout> {
    pub(crate) stdout: Option<Stdout>,
}

impl Context<Stdout> {
    pub fn production() -> Self {
        Context {
            stdout: Some(Stdout),
        }
    }
}

impl<Stdout> Context<Stdout>
where
    Stdout: Write + Send + Clone + 'static,
{
    pub(crate) fn spawn_stdout_relaying(
        &self,
        mut child_stdout: ChildStdout,
    ) -> JoinHandle<io::Result<Vec<u8>>> {
        let mut context = self.clone();
        thread::spawn(move || {
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
        })
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
    pub(crate) struct TestStdout(Arc<Mutex<Cursor<Vec<u8>>>>);

    impl TestStdout {
        fn new() -> TestStdout {
            TestStdout(Arc::new(Mutex::new(Cursor::new(Vec::new()))))
        }
    }

    impl Write for TestStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut lock = self.0.lock().unwrap();
            lock.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            let mut lock = self.0.lock().unwrap();
            lock.flush()
        }
    }

    impl Context<TestStdout> {
        pub(crate) fn test() -> Self {
            Context {
                stdout: Some(TestStdout::new()),
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
    }
}
