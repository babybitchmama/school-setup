//! Persistence for named custom styles: the `custom_styles_path` YAML file
//! declared in `Settings`. Kept deliberately dumb -- load the whole map,
//! mutate it in memory, write the whole map back -- since this file is
//! small and only ever touched by one daemon at a time.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single style's attributes, e.g. `{"stroke": "black", "fill": "none"}`.
/// Matches the shape of `Preset::attributes` so it plugs directly into
/// `normal::wrap_attributes_svg`.
pub type StyleAttributes = HashMap<String, String>;

/// name -> attributes
pub type StyleStore = HashMap<String, StyleAttributes>;

/// Loads the store from `path` (tilde-expanded). A missing file or a parse
/// failure both fall back to an empty store rather than erroring -- the
/// first `save-style` call will create it.
pub fn load(path: &str) -> StyleStore {
    let expanded = shellexpand::tilde(path).into_owned();
    fs::read_to_string(&expanded)
        .ok()
        .and_then(|contents| serde_yaml::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Writes the whole store back to `path`, creating parent directories if
/// needed.
pub fn save(path: &str, store: &StyleStore) -> std::io::Result<()> {
    let expanded = shellexpand::tilde(path).into_owned();
    if let Some(parent) = Path::new(&expanded).parent() {
        fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(store)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(expanded, yaml)
}

/// Inserts/overwrites `name` and persists immediately. Used by both
/// `save-style` (new or overwritten name) and `edit-style` (overwriting an
/// existing one after a live tweak in Inkscape).
pub fn upsert(path: &str, name: &str, attributes: StyleAttributes) -> std::io::Result<()> {
    let mut store = load(path);
    store.insert(name.to_string(), attributes);
    save(path, &store)
}
