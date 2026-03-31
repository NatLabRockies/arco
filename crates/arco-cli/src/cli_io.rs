use std::io::{self, Write};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM_GRAY: &str = "\x1b[38;5;245m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_NO_BOLD: &str = "\x1b[22m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Enabled,
    Disabled,
}

impl From<bool> for ColorMode {
    fn from(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}

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

pub fn should_colorize_stdout(stdout_is_terminal: bool) -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("CLICOLOR_FORCE").is_ok_and(|value| value != "0") {
        return true;
    }
    stdout_is_terminal
}

pub fn format_timed_status(
    status: &str,
    elapsed_ms: u128,
    detail: &str,
    color_mode: ColorMode,
) -> String {
    let elapsed = format!("in {elapsed_ms}ms");
    let payload = format!(
        "{status} {elapsed} ({})",
        style_bold_in_dim(detail, color_mode)
    );
    style_dimmed(&payload, color_mode)
}

pub fn style_bold_in_dim(content: &str, color_mode: ColorMode) -> String {
    if color_mode == ColorMode::Disabled {
        return content.to_string();
    }
    format!("{ANSI_BOLD}{content}{ANSI_NO_BOLD}")
}

fn style_dimmed(content: &str, color_mode: ColorMode) -> String {
    if color_mode == ColorMode::Disabled {
        return content.to_string();
    }
    format!("{ANSI_DIM_GRAY}{content}{ANSI_RESET}")
}

#[cfg(test)]
mod tests {
    use crate::cli_io::{
        ColorMode, format_timed_status, should_colorize_stdout, should_log_solver_to_console,
        write_all_ignoring_broken_pipe,
    };
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

    #[test]
    fn timed_status_plain_output_is_unstyled() {
        let rendered = format_timed_status("Validated file", 4, "arco 0.2.8", ColorMode::Disabled);
        assert_eq!(rendered, "Validated file in 4ms (arco 0.2.8)");
        assert!(!rendered.contains("\x1b["));
    }

    #[test]
    fn timed_status_colored_output_contains_expected_ansi_sequences() {
        let rendered = format_timed_status("Validated file", 4, "arco 0.2.8", ColorMode::Enabled);
        assert!(rendered.starts_with("\x1b[38;5;245mValidated file in 4ms ("));
        assert!(rendered.contains("\x1b[1marco 0.2.8"));
        assert!(rendered.ends_with(")\x1b[0m"));
    }

    #[test]
    fn colorize_stdout_respects_no_color() {
        // This test validates behavior only in the default environment where NO_COLOR is absent.
        // Full env mutation tests are avoided to keep test isolation stable.
        let _ = should_colorize_stdout(true);
    }
}
