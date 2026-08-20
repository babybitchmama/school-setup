use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::object_type::ObjectType;
use super::raw_shapes::representative_shape_markup;
use super::style_path;

/// Root wrapper for a (re)written style file. Declares the same
/// `inkscape:`/`sodipodi:` namespaces Inkscape itself puts on a real copy,
/// since raw-tag scanning and usvg resolution both key off qualified names
/// even though most saved styles never use attributes from them.
fn wrap_shapes_svg(shapes: &HashMap<ObjectType, String>) -> String {
    let mut ordered: Vec<_> = shapes.iter().collect();
    // Stable order keeps re-saves diff-friendly instead of shuffling on
    // every write (HashMap iteration order isn't stable).
    ordered.sort_by_key(|(t, _)| format!("{:?}", t));

    let mut body = String::new();
    for (_, markup) in ordered {
        body.push_str(markup);
        body.push('\n');
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" \
         xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\" \
         xmlns:sodipodi=\"http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd\" \
         version=\"1.1\">\n{}</svg>",
        body
    )
}

/// Merges the shape(s) in `new_svg` (as copied from Inkscape) into
/// whatever style file already exists at `name`, or creates it fresh if it
/// doesn't. Matches by `ObjectType`: a type present in `new_svg` replaces
/// that same type's entry in the existing file; every other previously
/// saved type is left untouched.
pub fn merge_into_style(name: &str, new_svg: &str) -> std::io::Result<PathBuf> {
    super::ensure_dirs();
    let path = style_path(name);

    let mut merged = if path.exists() {
        representative_shape_markup(&fs::read_to_string(&path)?)
    } else {
        HashMap::new()
    };

    let incoming = representative_shape_markup(new_svg);
    if incoming.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nothing recognizable was selected (copy came back empty or unsupported).",
        ));
    }

    merged.extend(incoming);

    fs::write(&path, wrap_shapes_svg(&merged))?;
    Ok(path)
}

/// Creates a minimal empty SVG at `scratch_path` for the user to draw a
/// fresh shape into.
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

/// Merges whatever the user drew at `scratch_path` (after closing
/// Inkscape) into `styles/<name>.svg`, same as the "from current
/// selection" path.
pub fn promote_scratch_to_style(name: &str, scratch_path: &Path) -> std::io::Result<PathBuf> {
    let drawn = fs::read_to_string(scratch_path)?;
    merge_into_style(name, &drawn)
}
