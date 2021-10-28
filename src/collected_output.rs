use crate::{config::Config, context::Context};
use std::io;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::{ChildStderr, ChildStdin, ChildStdout},
};

#[derive(Debug)]
pub(crate) struct Waiter {
    stdin: tokio::task::JoinHandle<io::Result<()>>,
    stdout: tokio::task::JoinHandle<io::Result<Option<Vec<u8>>>>,
    stderr: tokio::task::JoinHandle<io::Result<Option<Vec<u8>>>>,
    foo: tokio::task::JoinHandle<()>,
}

impl Waiter {
    fn spawn_standard_stream_handler(
        capture_stream: bool,
        mut source: impl AsyncRead + Send + Unpin + 'static,
        mut relay_sink: impl AsyncWrite + Send + Unpin + 'static,
    ) -> tokio::task::JoinHandle<io::Result<Option<Vec<u8>>>> {
        tokio::task::spawn(async move {
            let mut collected = if capture_stream {
                Some(Vec::new())
            } else {
                None
            };
            let buffer = &mut [0; 256];
            loop {
                let length = source.read(buffer).await?;
                if (length) == 0 {
                    break;
                }
                if let Some(collected) = &mut collected {
                    collected.extend(&buffer[..length]);
                }
                if !capture_stream {
                    relay_sink.write_all(&buffer[..length]).await?;
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
        Stdout: AsyncWrite + Send + Clone + Unpin + 'static,
        Stderr: AsyncWrite + Send + Clone + Unpin + 'static,
    {
        let config_stdin = config.stdin.clone();
        let stdin_join_handle = tokio::task::spawn(async move {
            AsyncWriteExt::write_all(&mut child_stdin, &config_stdin).await?;
            Ok(())
        });
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
            foo: tokio::task::spawn(std::future::ready(())),
        }
    }

    pub(crate) async fn join(self) -> io::Result<CollectedOutput> {
        self.foo.await.expect("fixme");
        self.stdin.await.expect("stdout relaying thread panicked")?;
        let stdout = self
            .stdout
            .await
            .expect("stdout relaying thread panicked")?;
        let stderr = self
            .stderr
            .await
            .expect("stderr relaying thread panicked")?;
        Ok(CollectedOutput { stdout, stderr })
    }
}

#[derive(Debug)]
pub(crate) struct CollectedOutput {
    pub(crate) stdout: Option<Vec<u8>>,
    pub(crate) stderr: Option<Vec<u8>>,
}
