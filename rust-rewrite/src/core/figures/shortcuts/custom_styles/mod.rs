//! Named "custom styles": small, hand-authored `.svg` files under
//! `~/.config/lesson-manager/figures/styles/`, each holding up to one
//! representative shape per object type (a rect, an ellipse, a path, ...).
//! Applying a style copies the matching shape's resolved style onto the
//! current Inkscape selection; saving a style captures the current
//! selection (or a freshly drawn one) back into one of these files.
//!
//! Style *resolution* (fill/stroke/opacity, with full CSS/inheritance
//! handling) goes through `usvg`. usvg flattens basic shapes into paths
//! and therefore can't tell us what tag a shape originally was, so a
//! separate lightweight pass over the raw XML (`raw_shapes`) identifies
//! shape types; the two are correlated by document order.

pub mod apply;
pub mod object_type;
pub mod parser;
pub mod raw_shapes;
pub mod save;

use std::fs;
use std::path::PathBuf;

pub fn styles_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde("~/.config/lesson-manager/figures/styles").into_owned())
}

pub fn ensure_dirs() {
    let _ = fs::create_dir_all(styles_dir());
    super::custom_objects::ensure_dir();
}

pub fn style_path(name: &str) -> PathBuf {
    styles_dir().join(format!("{}.svg", name))
}

pub fn list_style_names() -> Vec<String> {
    ensure_dirs();

    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(styles_dir()) {
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
