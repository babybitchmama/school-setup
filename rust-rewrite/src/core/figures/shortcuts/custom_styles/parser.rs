//! Resolving a style `.svg` file into `ObjectType -> StyleAttributes`,
//! using `usvg` for the actual fill/stroke/opacity computation (handles
//! `style=` vs. presentation attributes, inheritance from wrapping `<g>`,
//! `currentColor`, etc. -- all the things a hand-rolled attribute walk
//! kept missing) and `raw_shapes` purely to recover the original tag name
//! usvg throws away when it flattens shapes into paths.
//!
//! NOTE: usvg's exact struct/field names (`Fill`, `Stroke`, `Opacity`,
//! `StrokeWidth`, ...) have shifted across releases. If this doesn't
//! compile as-is against your locked version, check
//! `cargo doc --open -p usvg` (or docs.rs for that version) for the
//! current field names -- the logic/shape of this code should still be
//! right even if a field got renamed.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use usvg::{Node as UsvgNode, Options, Paint, Tree};

use super::object_type::ObjectType;
use super::raw_shapes::ordered_shape_types;

pub type StyleAttributes = HashMap<String, String>;

pub fn parse_style_file(path: &Path) -> HashMap<ObjectType, StyleAttributes> {
    match fs::read_to_string(path) {
        Ok(content) => parse_style_svg(&content),
        Err(e) => {
            println!("Could not read style file {}: {}", path.display(), e);
            HashMap::new()
        }
    }
}

pub fn parse_style_svg(content: &str) -> HashMap<ObjectType, StyleAttributes> {
    // Pass 1: what shape types appear, in document order (tag-name level,
    // usvg-independent).
    let raw_types = ordered_shape_types(content);

    // Pass 2: usvg's fully resolved tree, walked in the same depth-first
    // order, collecting one StyleAttributes per resolved Path node.
    let resolved = match Tree::from_str(content, &Options::default()) {
    Ok(tree) => resolved_path_styles_in_order(tree.root(), 1.0, usvg::Transform::identity()),
        Err(e) => {
            println!("usvg failed to parse style SVG: {}", e);
            Vec::new()
        }
    };

    if raw_types.len() != resolved.len() {
        println!(
            "Warning: raw tag scan found {} shape(s) but usvg resolved {} path(s) -- \
             they may be misaligned (likely a <text> element, which usvg handles \
             separately and this pass doesn't yet support). Pairing what overlaps.",
            raw_types.len(),
            resolved.len()
        );
    }

    let mut result = HashMap::new();
    for (object_type, attrs) in raw_types.into_iter().zip(resolved.into_iter()) {
        // First shape of a given type in the file wins, mirroring the
        // "one representative shape per type" convention.
        result.entry(object_type).or_insert(attrs);
    }
    result
}

/// Depth-first walk of the usvg tree collecting one `StyleAttributes` per
/// `Path` node. Both opacity and the geometric transform are accumulated
/// while descending through groups: opacity because it's not stored on
/// `Path` itself (see above), and transform because `stroke.width()` (and
/// dasharray) are reported in the path's *local* coordinate space, before
/// any scaling from a wrapping `<g transform="...">` or a `viewBox` that
/// doesn't match the document's stated width/height -- both of which
/// Inkscape writes routinely.
fn resolved_path_styles_in_order(
    group: &usvg::Group,
    inherited_opacity: f32,
    inherited_transform: usvg::Transform,
) -> Vec<StyleAttributes> {
    let mut out = Vec::new();
    let this_opacity = inherited_opacity * group.opacity().get();
    let this_transform = inherited_transform.pre_concat(group.transform());

    for node in group.children() {
        match node {
            UsvgNode::Path(path) => {
                out.push(path_to_style_attributes(path, this_opacity, this_transform))
            }
            UsvgNode::Group(child_group) => {
                out.extend(resolved_path_styles_in_order(
                    child_group,
                    this_opacity,
                    this_transform,
                ));
            }
            _ => {}
        }
    }
    out
}

/// Approximates a uniform scale factor out of a (possibly non-uniform,
/// rotated) affine transform, by averaging the magnitude of the
/// transformed x- and y-axis basis vectors. Good enough for the stroke
/// widths on simple hand-drawn style shapes; a true non-uniform scale
/// would technically turn a round stroke elliptical, which this doesn't
/// attempt to model.
fn approximate_scale(transform: usvg::Transform) -> f32 {
    let x_axis_len = (transform.sx.powi(2) + transform.ky.powi(2)).sqrt();
    let y_axis_len = (transform.kx.powi(2) + transform.sy.powi(2)).sqrt();
    (x_axis_len + y_axis_len) / 2.0
}

fn color_to_hex(color: usvg::Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

fn paint_to_css(paint: &Paint) -> String {
    match paint {
        Paint::Color(c) => color_to_hex(*c),
        Paint::LinearGradient(_) | Paint::RadialGradient(_) | Paint::Pattern(_) => {
            println!(
                "Note: gradient/pattern paints aren't supported by custom-styles yet; \
                 recording as 'none'."
            );
            "none".to_string()
        }
    }
}

fn path_to_style_attributes(
    path: &usvg::Path,
    opacity: f32,
    transform: usvg::Transform,
) -> StyleAttributes {
    let mut map = StyleAttributes::new();
    let scale = approximate_scale(transform);

    match path.fill() {
        Some(fill) => {
            map.insert("fill".to_string(), paint_to_css(fill.paint()));
            map.insert("fill-opacity".to_string(), fill.opacity().get().to_string());
            map.insert(
                "fill-rule".to_string(),
                match fill.rule() {
                    usvg::FillRule::NonZero => "nonzero",
                    usvg::FillRule::EvenOdd => "evenodd",
                }
                .to_string(),
            );
        }
        None => {
            map.insert("fill".to_string(), "none".to_string());
        }
    }

    match path.stroke() {
        Some(stroke) => {
            map.insert("stroke".to_string(), paint_to_css(stroke.paint()));
            map.insert(
                "stroke-width".to_string(),
                (stroke.width().get() * scale).to_string(),
            );
            map.insert(
                "stroke-opacity".to_string(),
                stroke.opacity().get().to_string(),
            );
            map.insert(
                "stroke-linecap".to_string(),
                match stroke.linecap() {
                    usvg::LineCap::Butt => "butt",
                    usvg::LineCap::Round => "round",
                    usvg::LineCap::Square => "square",
                }
                .to_string(),
            );
            map.insert(
                "stroke-linejoin".to_string(),
                match stroke.linejoin() {
                    usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => "miter",
                    usvg::LineJoin::Round => "round",
                    usvg::LineJoin::Bevel => "bevel",
                }
                .to_string(),
            );
            map.insert(
                "stroke-dasharray".to_string(),
                match stroke.dasharray() {
                    Some(dashes) => dashes
                        .iter()
                        .map(|d| (d * scale).to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    None => "none".to_string(),
                },
            );
        }
        None => {
            map.insert("stroke".to_string(), "none".to_string());
        }
    }

    map.insert("opacity".to_string(), opacity.to_string());

    map
}
