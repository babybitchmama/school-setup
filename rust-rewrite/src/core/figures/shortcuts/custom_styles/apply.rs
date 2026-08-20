//! Resolving "apply style X" into a concrete `<inkscape:clipboard>` paste
//! payload, based on what's currently selected in Inkscape.
//!
//! Detecting the selection's type is a raw tag-name check (no usvg needed
//! -- we're not resolving the selection's own style, just asking "what is
//! it"). Looking up the matching saved style goes through the usvg-backed
//! parser.

use super::object_type::ObjectType;
use super::parser::parse_style_file;
use super::raw_shapes::distinct_types_present;
use super::style_path;
use crate::core::figures::shortcuts::normal::wrap_attributes_svg;

pub fn resolve_style_svg(style_name: &str, copied_svg: &str) -> Result<String, String> {
    let types_present = distinct_types_present(copied_svg);

    let object_type: ObjectType = match types_present.as_slice() {
        [] => {
            return Err(
                "Nothing recognizable was selected (copy came back empty or unsupported)."
                    .to_string(),
            );
        }
        [only] => *only,
        _ => {
            return Err(format!(
                "Selection contains {} different shape types; apply-style only supports \
                 a single type at a time for now.",
                types_present.len()
            ));
        }
    };

    let style_map = parse_style_file(&style_path(style_name));
    let Some(attributes) = style_map.get(&object_type) else {
        return Err(format!(
            "Style '{}' has no saved {:?} style.",
            style_name, object_type
        ));
    };

    Ok(wrap_attributes_svg(attributes))
}
