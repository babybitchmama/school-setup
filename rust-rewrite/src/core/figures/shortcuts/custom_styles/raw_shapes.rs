//! Lightweight raw-XML shape identification, used two ways:
//!   1. During style-file parsing, to get the ordered list of tag-derived
//!      `ObjectType`s that we zip against usvg's resolved path list.
//!   2. Standalone, to detect what type(s) are present in a copied
//!      Inkscape selection -- no style resolution needed for that, just
//!      "what did the user select."

use roxmltree::{Document, Node};

use super::object_type::ObjectType;

/// Depth-first, document-order list of recognizable shape types. Skips
/// non-rendering containers (`defs`, `metadata`, `clipPath`,
/// `sodipodi:namedview`) so their contents don't get counted.
pub fn ordered_shape_types(svg_content: &str) -> Vec<ObjectType> {
    let Ok(doc) = Document::parse(svg_content) else {
        println!("raw_shapes: failed to parse SVG for tag scan");
        return Vec::new();
    };

    let mut out = Vec::new();
    walk(doc.root_element(), &mut out);
    out
}

fn is_ignored_container(tag: &str) -> bool {
    matches!(
        tag.rsplit(':').next().unwrap_or(tag),
        "defs" | "metadata" | "clipPath" | "namedview" | "mask" | "symbol"
    )
}

fn walk(node: Node, out: &mut Vec<ObjectType>) {
    if !node.is_element() {
        return;
    }

    let tag = node.tag_name().name();
    if is_ignored_container(tag) {
        return;
    }

    if let Some(object_type) = ObjectType::from_tag_name(tag) {
        out.push(object_type);
    }

    for child in node.children() {
        walk(child, out);
    }
}

/// Distinct types present, in first-seen order -- used by `apply` to
/// decide what a selection is (and to reject multi-type selections, for
/// now).
pub fn distinct_types_present(svg_content: &str) -> Vec<ObjectType> {
    let mut seen = Vec::new();
    for t in ordered_shape_types(svg_content) {
        if !seen.contains(&t) {
            seen.push(t);
        }
    }
    seen
}
