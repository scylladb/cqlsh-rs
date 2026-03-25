//! Built-in terminal pager using sapling-streampager.
//!
//! Provides a less-like pager with vertical/horizontal scrolling, search,
//! and proper ANSI color support. Uses termwiz for ANSI-aware rendering,
//! so horizontal scrolling works correctly with colored output.
//! Small output that fits the terminal is printed directly (Hybrid mode).

use std::io::Cursor;

use streampager::config::{InterfaceMode, WrappingMode};
use streampager::Pager;

/// Display content through the streampager pager.
///
/// Supports up/down scrolling, left/right horizontal scrolling for wide tables,
/// `/pattern` search, and correctly handles ANSI color codes.
/// If the content fits the terminal screen, it is printed directly (Hybrid mode).
///
/// An optional `title` is shown at the top of the pager (e.g., column names).
///
/// Returns `Err` if the pager fails to start; caller should fall back to direct print.
pub fn page_content(content: &str, title: &str) -> anyhow::Result<()> {
    let mut pager = Pager::new_using_system_terminal()?;
    pager.set_interface_mode(InterfaceMode::Hybrid);
    pager.set_wrapping_mode(WrappingMode::Unwrapped);
    pager.set_scroll_past_eof(false);
    pager.add_stream(Cursor::new(content.to_owned().into_bytes()), title)?;
    pager.run()?;
    Ok(())
}
