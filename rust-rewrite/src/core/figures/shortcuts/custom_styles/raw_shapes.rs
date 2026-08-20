//! Lightweight raw-XML shape identification, used two ways:
//!   1. During style-file parsing, to get the ordered list of tag-derived
//!      `ObjectType`s that we zip against usvg's resolved path list.
//!   2. Standalone, to detect what type(s) are present in a copied
//!      Inkscape selection -- no style resolution needed for that, just
//!      "what did the user select."

use super::object_type::ObjectType;
use roxmltree::{Document, Node};
use std::collections::HashMap;

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

/// Depth-first scan collecting the first shape of each recognized type,
/// paired with its exact original markup (sliced from `svg_content` via
/// roxmltree's byte ranges). Used by the style-merge save flow to graft a
/// freshly copied shape into an existing style file without disturbing
/// its other saved shapes.
pub fn representative_shape_markup(svg_content: &str) -> HashMap<ObjectType, String> {
    let Ok(doc) = Document::parse(svg_content) else {
        println!("raw_shapes: failed to parse SVG for markup extraction");
        return HashMap::new();
    };

    let mut out = HashMap::new();
    walk_markup(doc.root_element(), svg_content, &mut out);
    out
}

fn walk_markup(node: Node, source: &str, out: &mut HashMap<ObjectType, String>) {
    if !node.is_element() {
        return;
    }

    let tag = node.tag_name().name();
    if is_ignored_container(tag) {
        return;
    }

    if let Some(object_type) = ObjectType::from_tag_name(tag) {
        // First shape of a given type wins, same convention as
        // `parser::parse_style_svg` -- don't descend into it either, a
        // matched shape's markup is already captured whole.
        out.entry(object_type)
            .or_insert_with(|| source[node.range()].to_string());
        return;
    }

    for child in node.children() {
        walk_markup(child, source, out);
    }
}
