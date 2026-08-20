//! Named "custom objects": full, ready-to-paste `.svg` files under
//! `~/.config/lesson-manager/figures/objects/`, e.g. a manifold diagram or
//! a commutative-square template.
//!
//! Unlike custom styles (which only capture one representative shape's
//! *style* attributes and get remixed onto whatever's currently selected),
//! an object's entire SVG content is copied verbatim onto the clipboard
//! and pasted in place with Ctrl+Alt+V, so a multi-shape object keeps the
//! internal layout it was saved with.

use std::fs;
use std::path::PathBuf;

pub fn objects_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde("~/.config/lesson-manager/figures/objects").into_owned())
}

pub fn ensure_dir() {
    let _ = fs::create_dir_all(objects_dir());
}

pub fn object_path(name: &str) -> PathBuf {
    objects_dir().join(format!("{}.svg", name))
}

/// Sorted list of object names (file stems) available to paste, mirroring
/// `custom_styles::list_style_names`.
pub fn list_object_names() -> Vec<String> {
    ensure_dir();

    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(objects_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// Reads an object's raw SVG content, unmodified, ready to hand to the
/// clipboard as-is.
pub fn read_object_svg(name: &str) -> std::io::Result<String> {
    fs::read_to_string(object_path(name))
}

/// Writes `svg_content` (as read back from Inkscape's native clipboard
/// target after a real Ctrl+C) directly to `objects/<name>.svg`.
pub fn save_from_clipboard_svg(name: &str, svg_content: &str) -> std::io::Result<PathBuf> {
    ensure_dir();
    let path = object_path(name);
    fs::write(&path, svg_content)?;
    Ok(path)
}
