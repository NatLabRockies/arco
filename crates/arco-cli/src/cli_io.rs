use std::io::{self, Write};

pub fn write_all_ignoring_broken_pipe<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    ignore_broken_pipe(writer.write_all(bytes))?;
    ignore_broken_pipe(writer.flush())
}

fn ignore_broken_pipe(result: io::Result<()>) -> io::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn write_stdout(bytes: &[u8]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_all_ignoring_broken_pipe(&mut handle, bytes)
}

pub fn write_stdout_line(line: &str) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_all_ignoring_broken_pipe(&mut handle, line.as_bytes())?;
    write_all_ignoring_broken_pipe(&mut handle, b"\n")
}

pub fn should_log_solver_to_console(verbose: u8, stdout_is_terminal: bool) -> bool {
    verbose >= 2 && stdout_is_terminal
}

#[cfg(test)]
mod tests {
    use crate::cli_io::{should_log_solver_to_console, write_all_ignoring_broken_pipe};
    use std::io::{self, Write};

    struct BrokenPipeWriter;

    impl Write for BrokenPipeWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("disk full"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn ignores_broken_pipe_errors() -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = BrokenPipeWriter;
        write_all_ignoring_broken_pipe(&mut writer, b"hello")?;
        Ok(())
    }

    #[test]
    fn returns_other_io_errors() {
        let mut writer = FailingWriter;
        let error = write_all_ignoring_broken_pipe(&mut writer, b"hello")
            .expect_err("non-broken-pipe errors should be returned");
        assert_eq!(error.kind(), io::ErrorKind::Other);
    }

    #[test]
    fn only_enables_solver_console_logging_for_double_verbose_terminals() {
        assert!(should_log_solver_to_console(2, true));
        assert!(!should_log_solver_to_console(1, true));
        assert!(!should_log_solver_to_console(2, false));
    }
}
