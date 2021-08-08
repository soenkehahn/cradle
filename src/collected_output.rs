use crate::{config::Config, context::Context};
use std::{
    io::{self, Read, Write},
    process::{ChildStderr, ChildStdin, ChildStdout},
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub(crate) struct Waiter {
    stdin: JoinHandle<io::Result<()>>,
    stdout: JoinHandle<io::Result<Option<Vec<u8>>>>,
    stderr: JoinHandle<io::Result<Vec<u8>>>,
}

impl Waiter {
    pub(crate) fn spawn_standard_stream_relaying<Stdout, Stderr>(
        context: &Context<Stdout, Stderr>,
        config: &Config,
        mut child_stdin: ChildStdin,
        mut child_stdout: ChildStdout,
        mut child_stderr: ChildStderr,
    ) -> Self
    where
        Stdout: Write + Send + Clone + 'static,
        Stderr: Write + Send + Clone + 'static,
    {
        let config_stdin = config.stdin.clone();
        let stdin_join_handle = thread::spawn(move || -> io::Result<()> {
            child_stdin.write_all(&config_stdin)?;
            Ok(())
        });
        let mut context_clone = context.clone();
        let capture_stdout = config.capture_stdout;
        let stdout_join_handle = thread::spawn(move || -> io::Result<Option<Vec<u8>>> {
            let mut collected_stdout = if capture_stdout {
                Some(Vec::new())
            } else {
                None
            };
            let buffer = &mut [0; 256];
            loop {
                let length = child_stdout.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                if let Some(collected_stdout) = &mut collected_stdout {
                    collected_stdout.extend(&buffer[..length]);
                }
                if !capture_stdout {
                    context_clone.stdout.write_all(&buffer[..length])?;
                }
            }
            Ok(collected_stdout)
        });
        let mut context_clone = context.clone();
        let capture_stderr = config.capture_stderr;
        let stderr_join_handle = thread::spawn(move || -> io::Result<Vec<u8>> {
            let mut collected_stderr = Vec::new();
            let buffer = &mut [0; 256];
            loop {
                let length = child_stderr.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                collected_stderr.extend(&buffer[..length]);
                if !capture_stderr {
                    context_clone.stderr.write_all(&buffer[..length])?;
                }
            }
            Ok(collected_stderr)
        });
        Waiter {
            stdin: stdin_join_handle,
            stdout: stdout_join_handle,
            stderr: stderr_join_handle,
        }
    }

    pub(crate) fn join(self) -> io::Result<CollectedOutput> {
        self.stdin
            .join()
            .expect("stdout relaying thread panicked")?;
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
    pub(crate) stderr: Vec<u8>,
}
