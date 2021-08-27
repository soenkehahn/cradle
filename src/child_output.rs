use crate::{
    collected_output::Waiter, config::Config, context::Context, error::Error, output::Output,
};
use std::{
    ffi::OsString,
    io::Write,
    process::{Command, ExitStatus, Stdio},
    sync::Arc,
};

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct ChildOutput {
    pub(crate) stdout: Option<Vec<u8>>,
    pub(crate) stderr: Option<Vec<u8>>,
    pub(crate) exit_status: ExitStatus,
}

impl ChildOutput {
    pub(crate) fn run_child_process_output<Stdout, Stderr, T>(
        context: Context<Stdout, Stderr>,
        mut config: Config,
    ) -> Result<T, Error>
    where
        Stdout: Write + Clone + Send + 'static,
        Stderr: Write + Clone + Send + 'static,
        T: Output,
    {
        <T as Output>::configure(&mut config);
        let result = ChildOutput::run_child_process(context, &config)?;
        T::from_run_result(&config, result)
    }

    fn run_child_process<Stdout, Stderr>(
        mut context: Context<Stdout, Stderr>,
        config: &Config,
    ) -> Result<Self, Error>
    where
        Stdout: Write + Clone + Send + 'static,
        Stderr: Write + Clone + Send + 'static,
    {
        let (executable, arguments) = Self::parse_input(config.arguments.clone())?;
        if config.log_command {
            writeln!(context.stderr, "+ {}", config.full_command())
                .map_err(|error| Error::command_io_error(config, error))?;
        }
        let mut command = Command::new(&executable);
        command.args(arguments);
        for (key, value) in &config.added_environment_variables {
            command.env(key, value);
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(working_directory) = &config.working_directory {
            command.current_dir(working_directory);
        }
        let mut child = command.spawn().map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::FileNotFound {
                    executable,
                    source: Arc::new(error),
                }
            } else {
                Error::command_io_error(config, error)
            }
        })?;
        let waiter = Waiter::spawn_standard_stream_relaying(
            &context,
            config,
            child.stdin.take().expect("child process should have stdin"),
            child
                .stdout
                .take()
                .expect("child process should have stdout"),
            child
                .stderr
                .take()
                .expect("child process should have stderr"),
        );
        let exit_status = child
            .wait()
            .map_err(|error| Error::command_io_error(config, error))?;
        let collected_output = waiter
            .join()
            .map_err(|error| Error::command_io_error(config, error))?;
        Self::check_exit_status(config, exit_status)?;
        Ok(Self {
            stdout: collected_output.stdout,
            stderr: collected_output.stderr,
            exit_status,
        })
    }

    fn parse_input(
        input: Vec<OsString>,
    ) -> Result<(OsString, impl Iterator<Item = OsString>), Error> {
        let mut words = input.into_iter();
        {
            match words.next() {
                None => Err(Error::NoArgumentsGiven),
                Some(command) => Ok((command, words)),
            }
        }
    }

    fn check_exit_status(config: &Config, exit_status: ExitStatus) -> Result<(), Error> {
        if config.error_on_non_zero_exit_code && !exit_status.success() {
            Err(Error::NonZeroExitCode {
                full_command: config.full_command(),
                exit_status,
            })
        } else {
            Ok(())
        }
    }
}
