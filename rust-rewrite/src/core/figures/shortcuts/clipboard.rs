use std::io::Write;
use std::process::{Command, Stdio};

use super::constants::CLIPBOARD_TARGET;

/// Pushes `content` onto the X clipboard tagged with Inkscape's native SVG
/// clipboard target — mirrors Castel's `clipboard.py` (which also just
/// shells out to `xclip`) and this project's existing `copy_template_to_clipboard`.
pub fn copy(content: &str) -> std::io::Result<()> {
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-target")
        .arg(CLIPBOARD_TARGET)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}
