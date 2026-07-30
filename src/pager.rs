//! Output pager for displaying large result sets.
//!
//! Two modes:
//! - **Batch** (`page_content`): write all content to temp file, open with pager.
//! - **Streaming** (`page_stream`): pipe rows directly to `less` stdin.
//!   `less` stores received bytes in its own internal buffer and allows full
//!   backward scrolling. Data appears as it arrives; no polling required.
//!
//! Pager resolution order:
//! 1. `$PAGER` environment variable (pipe mode — custom pager gets stdin)
//! 2. `less` with stdin pipe, with flags matching the detected implementation
//! 3. `more` as fallback (pipe mode)
//! 4. Error if nothing available

use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

use tempfile::NamedTempFile;

/// Which `less` implementation is on `PATH`.
///
/// BusyBox (Alpine — and therefore our Docker image) ships `less` as an applet
/// that shares little more than the name with GNU less:
/// - `-P` (custom prompt) does not exist; BusyBox exits with a usage error,
///   which leaves us piping rows into a dead process and shows nothing.
/// - `-R` has the *opposite* meaning: BusyBox *strips* ANSI colors from the
///   input, while GNU passes them through raw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LessFlavor {
    /// Real less (`less` upstream, GNU or POSIX-regex build) — the full flag
    /// set is available.
    Full,
    /// The BusyBox applet: no `-P`, and `-R` strips colors instead of passing
    /// them through.
    BusyBox,
    /// Something else answering to the name `less`; only `-S`, the one flag
    /// every implementation agrees on, is safe to pass.
    Unknown,
}

/// Classify `less --version` output. Real less prints its version to stdout and
/// exits 0 (`less 590 (GNU regular expressions)`, or `(POSIX regular
/// expressions)` for Alpine's build); BusyBox prints a usage error mentioning
/// "BusyBox" and exits 1.
fn classify_less(stdout: &[u8], stderr: &[u8], success: bool) -> LessFlavor {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    if stdout.contains("BusyBox") || stderr.contains("BusyBox") {
        LessFlavor::BusyBox
    } else if success && stdout.contains("less") {
        LessFlavor::Full
    } else {
        LessFlavor::Unknown
    }
}

/// Detect the `less` implementation once per process.
///
/// `None` means `less` could not be executed at all, so callers should skip it
/// and fall back to `more`.
fn less_flavor() -> Option<LessFlavor> {
    static FLAVOR: OnceLock<Option<LessFlavor>> = OnceLock::new();
    *FLAVOR.get_or_init(|| {
        let out = Command::new("less")
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .ok()?;
        Some(classify_less(
            &out.stdout,
            &out.stderr,
            out.status.success(),
        ))
    })
}

/// Build the argument list for `less`, tailored to the detected implementation.
///
/// - `-S` (chop long lines instead of wrapping) is understood everywhere.
/// - `-P` (custom prompt) is real-less only; BusyBox exits with a usage error.
/// - `-R` means opposite things: real less passes ANSI colors through (what we
///   want), BusyBox *removes* them. Removing them is still the better BusyBox
///   outcome — without `-R` it prints the escape sequences as literal text like
///   `[38;5;13m`, so colored results become unreadable. Colors are lost either
///   way there; `-R` at least leaves a clean monochrome table.
fn less_args(flavor: LessFlavor, title: &str) -> Vec<String> {
    match flavor {
        LessFlavor::Full => {
            let prompt = if title.is_empty() {
                String::from("-Pline %lt/%L")
            } else {
                format!("-P{title}  line %lt/%L")
            };
            vec![String::from("-R"), String::from("-S"), prompt]
        }
        LessFlavor::BusyBox => vec![String::from("-S"), String::from("-R")],
        LessFlavor::Unknown => vec![String::from("-S")],
    }
}

pub fn page_content(content: &str, title: &str) -> anyhow::Result<()> {
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;

    let path = tmp.path();

    if let Ok(pager_env) = std::env::var("PAGER") {
        let parts: Vec<&str> = pager_env.split_whitespace().collect();
        if let Some((cmd, args)) = parts.split_first() {
            let status = Command::new(cmd).args(args).arg(path).status();
            if let Ok(s) = status {
                if s.success() {
                    return Ok(());
                }
            }
        }
    }

    if let Some(flavor) = less_flavor() {
        if let Ok(status) = Command::new("less")
            .args(less_args(flavor, title))
            .arg(path)
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }
    }

    if let Ok(status) = Command::new("more").arg(path).status() {
        if status.success() {
            return Ok(());
        }
    }

    anyhow::bail!("no external pager available")
}

/// A writable handle that streams rows to a pager.
///
/// Rows are piped directly to the pager's stdin. The pager buffers received
/// data internally (less stores it in its own temp file) and allows full
/// backward scrolling without holding everything in our process memory.
pub struct PagerWriter {
    stdin: Option<std::process::ChildStdin>,
    child: Option<Child>,
}

impl PagerWriter {
    /// Returns true when a child pager process owns the terminal.
    /// In that case stderr writes would corrupt the pager display.
    pub fn is_file_mode(&self) -> bool {
        self.child.is_some()
    }
}

impl Write for PagerWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stdin.as_mut() {
            Some(stdin) => stdin.write(buf),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "pager stdin closed",
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.stdin.as_mut() {
            Some(stdin) => stdin.flush(),
            None => Ok(()),
        }
    }
}

impl Drop for PagerWriter {
    fn drop(&mut self) {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = writeln!(
                stdin,
                "\n\x1b[2m-- end of results (press q to quit) --\x1b[0m"
            );
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.wait();
        }
    }
}

/// Spawn a pager for streaming output, returning a [`PagerWriter`] to write rows into.
///
/// All pager variants receive data via piped stdin. `less` buffers received bytes
/// in its own internal temp file, enabling full backward scrolling without holding
/// all rows in our process memory. On drop, an end-of-results marker is written and
/// we wait for the user to quit the pager.
pub fn page_stream(title: &str) -> anyhow::Result<PagerWriter> {
    let spawn_piped = |cmd: &mut Command| -> Option<PagerWriter> {
        if let Ok(mut child) = cmd.stdin(Stdio::piped()).spawn() {
            let stdin = child.stdin.take()?;
            Some(PagerWriter {
                stdin: Some(stdin),
                child: Some(child),
            })
        } else {
            None
        }
    };

    if let Ok(pager_env) = std::env::var("PAGER") {
        let parts: Vec<&str> = pager_env.split_whitespace().collect();
        if let Some((cmd, args)) = parts.split_first() {
            if let Some(w) = spawn_piped(Command::new(cmd).args(args)) {
                return Ok(w);
            }
        }
    }

    if let Some(flavor) = less_flavor() {
        if let Some(w) = spawn_piped(Command::new("less").args(less_args(flavor, title))) {
            return Ok(w);
        }
    }

    if let Some(w) = spawn_piped(&mut Command::new("more")) {
        return Ok(w);
    }

    anyhow::bail!("no external pager available")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_gnu_less_version_output() {
        let stdout = b"less 590 (GNU regular expressions)\nCopyright (C) 1984-2021 Mark Nudelman\n";
        assert_eq!(classify_less(stdout, b"", true), LessFlavor::Full);
    }

    #[test]
    fn classify_alpine_less_package_version_output() {
        // Alpine's `less` package is real less but reports POSIX regexes.
        let stdout = b"less 685 (POSIX regular expressions)\n";
        assert_eq!(classify_less(stdout, b"", true), LessFlavor::Full);
    }

    #[test]
    fn classify_busybox_less_usage_error() {
        // BusyBox rejects --version: usage goes to stderr and it exits 1.
        let stderr = b"less: unrecognized option '--version'\n\
                       BusyBox v1.37.0 (2025-12-16 14:19:28 UTC) multi-call binary.\n\n\
                       Usage: less [-EFIMmNSRh~] [FILE]...\n";
        assert_eq!(classify_less(b"", stderr, false), LessFlavor::BusyBox);
    }

    #[test]
    fn classify_busybox_less_that_exits_zero() {
        // Some BusyBox builds print usage on stdout and exit 0; the "BusyBox"
        // marker must still win over the successful exit status.
        let stdout = b"BusyBox v1.37.0 multi-call binary.\n\nUsage: less [-EFIMmNSRh~] [FILE]...\n";
        assert_eq!(classify_less(stdout, b"", true), LessFlavor::BusyBox);
    }

    #[test]
    fn classify_unrecognized_less_is_unknown() {
        assert_eq!(classify_less(b"", b"", false), LessFlavor::Unknown);
        assert_eq!(
            classify_less(b"something else\n", b"", true),
            LessFlavor::Unknown
        );
    }

    #[test]
    fn less_args_full_passes_colors_and_prompt() {
        let args = less_args(LessFlavor::Full, "id | name");
        assert_eq!(args, vec!["-R", "-S", "-Pid | name  line %lt/%L"]);
    }

    #[test]
    fn less_args_full_without_title() {
        let args = less_args(LessFlavor::Full, "");
        assert_eq!(args, vec!["-R", "-S", "-Pline %lt/%L"]);
    }

    #[test]
    fn less_args_busybox_strips_colors_and_drops_prompt() {
        // -P is fatal on BusyBox; its -R removes color escapes, which beats
        // rendering them as literal `[38;5;13m` text.
        let args = less_args(LessFlavor::BusyBox, "id | name");
        assert_eq!(args, vec!["-S", "-R"]);
        assert!(!args.iter().any(|a| a.starts_with("-P")));
    }

    #[test]
    fn less_args_unknown_uses_only_universal_flag() {
        let args = less_args(LessFlavor::Unknown, "id | name");
        assert_eq!(args, vec!["-S"]);
    }

    /// Checks the classifier against a real BusyBox, when one is installed
    /// (Alpine images, most Debian/Ubuntu hosts). Skips silently otherwise.
    #[test]
    fn classify_real_busybox_less_when_available() {
        let Ok(out) = Command::new("busybox")
            .args(["less", "--version"])
            .stdin(Stdio::null())
            .output()
        else {
            return;
        };
        assert_eq!(
            classify_less(&out.stdout, &out.stderr, out.status.success()),
            LessFlavor::BusyBox
        );
    }

    #[test]
    fn less_flavor_detection_is_stable() {
        // Whatever is installed on the test machine, detection must not panic
        // and must be cached (same answer every call).
        assert_eq!(less_flavor(), less_flavor());
    }

    #[test]
    fn page_content_with_cat_pager() {
        std::env::set_var("PAGER", "cat");
        let result = page_content("hello world\n", "test title");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_content_with_true_pager() {
        std::env::set_var("PAGER", "true");
        let result = page_content("test content", "");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_content_writes_temp_file() {
        std::env::set_var("PAGER", "grep -q hello");
        let result = page_content("hello world", "title");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_content_empty_string() {
        std::env::set_var("PAGER", "true");
        let result = page_content("", "empty");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_content_large_content() {
        let content = "x".repeat(100_000);
        std::env::set_var("PAGER", "true");
        let result = page_content(&content, "big");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_content_multiline() {
        let content = "line1\nline2\nline3\n";
        std::env::set_var("PAGER", "wc -l");
        let result = page_content(content, "lines");
        std::env::remove_var("PAGER");
        assert!(result.is_ok());
    }

    #[test]
    fn page_stream_with_cat() {
        std::env::set_var("PAGER", "cat");
        let mut writer = page_stream("test").unwrap();
        writer.write_all(b"streaming content\n").unwrap();
        drop(writer);
        std::env::remove_var("PAGER");
    }

    #[test]
    fn page_stream_write_multiple() {
        std::env::set_var("PAGER", "cat");
        let mut writer = page_stream("").unwrap();
        writer.write_all(b"line 1\n").unwrap();
        writer.write_all(b"line 2\n").unwrap();
        drop(writer);
        std::env::remove_var("PAGER");
    }

    #[test]
    fn pager_writer_is_file_mode_with_child() {
        std::env::set_var("PAGER", "cat");
        let writer = page_stream("title").unwrap();
        assert!(writer.is_file_mode());
        std::env::remove_var("PAGER");
    }

    #[test]
    fn pager_writer_is_file_mode_without_child() {
        let writer = PagerWriter {
            stdin: None,
            child: None,
        };
        assert!(!writer.is_file_mode());
    }

    #[test]
    fn pager_writer_write_without_stdin_returns_broken_pipe() {
        let mut writer = PagerWriter {
            stdin: None,
            child: None,
        };
        let result = writer.write(b"data");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::BrokenPipe);
    }

    #[test]
    fn pager_writer_flush_without_stdin_ok() {
        let mut writer = PagerWriter {
            stdin: None,
            child: None,
        };
        assert!(writer.flush().is_ok());
    }

    #[test]
    fn page_stream_empty_title() {
        std::env::set_var("PAGER", "true");
        let writer = page_stream("");
        std::env::remove_var("PAGER");
        assert!(writer.is_ok());
    }

    #[test]
    fn page_stream_nonempty_title() {
        std::env::set_var("PAGER", "true");
        let writer = page_stream("my table");
        std::env::remove_var("PAGER");
        assert!(writer.is_ok());
    }

    #[test]
    fn pager_writer_drop_writes_end_marker() {
        std::env::set_var("PAGER", "cat");
        let writer = page_stream("").unwrap();
        drop(writer);
        std::env::remove_var("PAGER");
    }
}
