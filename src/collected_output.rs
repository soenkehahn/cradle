use crate::{config::Config, context::Context};
use std::{
    io::{self, Read, Write},
    process::{ChildStderr, ChildStdin, ChildStdout},
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub(crate) struct Waiter {
    stdin: Option<JoinHandle<io::Result<()>>>,
    stdout: JoinHandle<io::Result<Option<Vec<u8>>>>,
    stderr: JoinHandle<io::Result<Option<Vec<u8>>>>,
}

impl Waiter {
    fn spawn_standard_stream_handler(
        capture_stream: bool,
        mut source: impl Read + Send + 'static,
        mut relay_sink: impl Write + Send + 'static,
    ) -> JoinHandle<io::Result<Option<Vec<u8>>>> {
        thread::spawn(move || -> io::Result<Option<Vec<u8>>> {
            let mut collected = if capture_stream {
                Some(Vec::new())
            } else {
                None
            };
            let buffer = &mut [0; 256];
            loop {
                let length = source.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                if let Some(collected) = &mut collected {
                    collected.extend(&buffer[..length]);
                }
                if !capture_stream {
                    relay_sink.write_all(&buffer[..length])?;
                }
            }
            Ok(collected)
        })
    }

    pub(crate) fn spawn_standard_stream_relaying<Stdout, Stderr>(
        context: &Context<Stdout, Stderr>,
        config: &Config,
        mut child_stdin: ChildStdin,
        child_stdout: ChildStdout,
        child_stderr: ChildStderr,
    ) -> Self
    where
        Stdout: Write + Send + Clone + 'static,
        Stderr: Write + Send + Clone + 'static,
    {
        let stdin_join_handle = match config.stdin.clone() {
            Some(config_stdin) => Some(thread::spawn(move || -> io::Result<()> {
                child_stdin.write_all(&config_stdin)?;
                Ok(())
            })),
            None => None,
        };
        let stdout_join_handle = Self::spawn_standard_stream_handler(
            config.capture_stdout,
            child_stdout,
            context.stdout.clone(),
        );
        let stderr_join_handle = Self::spawn_standard_stream_handler(
            config.capture_stderr,
            child_stderr,
            context.stderr.clone(),
        );
        Waiter {
            stdin: stdin_join_handle,
            stdout: stdout_join_handle,
            stderr: stderr_join_handle,
        }
    }

    pub(crate) fn join(self) -> io::Result<CollectedOutput> {
        if let Some(stdin) = self.stdin {
            stdin.join().expect("stdout relaying thread panicked")?;
        }
        Ok(CollectedOutput {
            stdout: self
                .stdout
                .join()
                .expect("stdout relaying thread panicked")?,
            stderr: self
                .stderr
                .join()
                .expect("stderr relaying thread panicked")?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct CollectedOutput {
    pub(crate) stdout: Option<Vec<u8>>,
    pub(crate) stderr: Option<Vec<u8>>,
}
