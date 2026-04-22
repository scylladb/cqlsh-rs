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
//! 2. `less` with stdin pipe (`less -R -S`)
//! 3. `more` as fallback (pipe mode)
//! 4. Error if nothing available

use std::io::Write;
use std::process::{Child, Command, Stdio};

use tempfile::NamedTempFile;

pub fn page_content(content: &str, _title: &str) -> anyhow::Result<()> {
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

    if let Ok(status) = Command::new("less").args(["-R", "-S"]).arg(path).status() {
        if status.success() {
            return Ok(());
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

    let prompt = if title.is_empty() {
        String::from("-Pline %lt/%L")
    } else {
        format!("-P{title}  line %lt/%L")
    };

    if let Some(w) = spawn_piped(Command::new("less").args(["-R", "-S", &prompt])) {
        return Ok(w);
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
        std::env::set_var("PAGER", "true");
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
