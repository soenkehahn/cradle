use crate::{Config, Context};
use std::{
    io::{self, Read, Write},
    process::{ChildStderr, ChildStdout},
    thread::{self, JoinHandle},
};

pub(crate) struct Waiter {
    stdout: JoinHandle<io::Result<Vec<u8>>>,
    stderr: JoinHandle<io::Result<Vec<u8>>>,
}

impl Waiter {
    pub(crate) fn spawn_standard_stream_relaying<Stdout, Stderr>(
        context: &Context<Stdout, Stderr>,
        config: Config,
        mut child_stdout: ChildStdout,
        mut child_stderr: ChildStderr,
    ) -> Self
    where
        Stdout: Write + Send + Clone + 'static,
        Stderr: Write + Send + Clone + 'static,
    {
        let mut context_clone = context.clone();
        let config_clone = config.clone();
        let stdout_join_handle = thread::spawn(move || {
            let mut collected_stdout = Vec::new();
            let buffer = &mut [0; 256];
            loop {
                let length = child_stdout.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                if config_clone.relay_stdout {
                    context_clone.stdout.write_all(&buffer[..length])?;
                }
                collected_stdout.extend(&buffer[..length]);
            }
            Ok(collected_stdout)
        });
        let mut context_clone = context.clone();
        let stderr_join_handle = thread::spawn(move || {
            let mut collected_stderr = Vec::new();
            let buffer = &mut [0; 256];
            loop {
                let length = child_stderr.read(buffer)?;
                if (length) == 0 {
                    break;
                }
                if config.relay_stderr {
                    context_clone.stderr.write_all(&buffer[..length])?;
                }
                collected_stderr.extend(&buffer[..length]);
            }
            Ok(collected_stderr)
        });
        Waiter {
            stdout: stdout_join_handle,
            stderr: stderr_join_handle,
        }
    }

    pub(crate) fn join(self) -> io::Result<CollectedOutput> {
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

pub(crate) struct CollectedOutput {
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}
