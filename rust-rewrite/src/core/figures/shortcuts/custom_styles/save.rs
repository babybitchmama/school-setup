//! Persisting a custom style to disk, either by writing back real Inkscape
//! clipboard markup (the "from current selection" path) or by promoting a
//! scratch file the user drew from a blank canvas (the "start blank" path).

use std::fs;
use std::path::{Path, PathBuf};

use super::style_path;

/// Writes `svg_content` (as read back from Inkscape's native clipboard
/// target after a real Ctrl+C) directly to `styles/<name>.svg`. Inkscape's
/// rich copy format already contains real `<rect>`/`<path>`/... elements
/// with their `style` attributes intact, so no transformation is needed.
pub fn save_from_clipboard_svg(name: &str, svg_content: &str) -> std::io::Result<PathBuf> {
    super::ensure_dirs();
    let path = style_path(name);
    fs::write(&path, svg_content)?;
    Ok(path)
}

/// Creates a minimal empty SVG at `scratch_path` for the user to draw a
/// fresh style into.
pub fn create_blank_scratch_svg(scratch_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = scratch_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let minimal_svg = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" "#,
        r#"xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" version="1.1"></svg>"#,
    );

    fs::write(scratch_path, minimal_svg)
}

/// Copies whatever the user saved at `scratch_path` (after closing
/// Inkscape) into `styles/<name>.svg`.
pub fn promote_scratch_to_style(name: &str, scratch_path: &Path) -> std::io::Result<PathBuf> {
    super::ensure_dirs();
    let dest = style_path(name);
    fs::copy(scratch_path, &dest)?;
    Ok(dest)
}
