use std::collections::{HashMap, HashSet};

use super::config::StylesConfig;

type Style = HashMap<String, String>;

struct ComputedStyle {
    attributes: Style,
    marker_width: Option<f64>,
}

/// Builds the `<inkscape:clipboard>` SVG snippet for a set of simultaneously
/// held keys, mirroring Castel's `paste_style`. Returns `None` when the keys
/// don't resolve to anything (both fill and stroke end up "none").
pub fn build_style_svg(styles: &StylesConfig, pressed: &[String]) -> Option<String> {
    if pressed.len() == 1 {
        if let Some(preset) = styles.find_preset(&pressed[0]) {
            let computed = ComputedStyle {
                attributes: preset.attributes.clone(),
                marker_width: None,
            };
            return Some(wrap_style_svg(&computed));
        }
    }

    let computed = compute_modifier_style(styles, pressed);
    let stroke_none = computed.attributes.get("stroke").map(String::as_str) == Some("none");
    let fill_none = computed.attributes.get("fill").map(String::as_str) == Some("none");
    if stroke_none && fill_none {
        return None;
    }
    Some(wrap_style_svg(&computed))
}

/// Wraps a raw attribute map -- e.g. one loaded from a saved custom style
/// -- into the same `<inkscape:clipboard>` SVG snippet used for built-in
/// chords and presets, so custom styles paste the same way they do.
pub fn wrap_attributes_svg(attributes: &HashMap<String, String>) -> String {
    let computed = ComputedStyle {
        attributes: attributes.clone(),
        marker_width: None,
    };
    wrap_style_svg(&computed)
}

/// Resolves the actual set of modifier keys a chord means, expanding any
/// key that's a `combo` trigger into the keys it stands in for. A combo
/// key held alongside literal modifier keys just adds its keys to the same
/// set -- e.g. holding a "grey+dashed+stroke" combo plus `g` (thick) still
/// layers thick on top, since everything downstream only cares about set
/// membership. Expansion is one level: a key inside a combo's `keys` list
/// is taken literally, even if it happens to also be another combo's
/// trigger.
fn expand_combos<'a>(styles: &'a StylesConfig, pressed: &'a [String]) -> HashSet<&'a str> {
    let mut held = HashSet::new();
    for p in pressed {
        match styles.find_combo(p) {
            Some(combo) => held.extend(combo.keys.iter().map(String::as_str)),
            None => {
                held.insert(p.as_str());
            }
        }
    }
    held
}

fn compute_modifier_style(styles: &StylesConfig, pressed: &[String]) -> ComputedStyle {
    let held = expand_combos(styles, pressed);
    let settings = &styles.settings;

    let stroke_keys: HashSet<&str> = styles
        .modifiers
        .strokes
        .iter()
        .map(|m| m.key.as_str())
        .collect();
    let dash_keys: HashSet<&str> = styles
        .modifiers
        .dashes
        .iter()
        .map(|m| m.key.as_str())
        .collect();
    let arrow_keys: HashSet<&str> = styles
        .modifiers
        .arrows
        .iter()
        .map(|m| m.key.as_str())
        .collect();

    let mut style = Style::new();
    style.insert("stroke-opacity".to_string(), "1".to_string());

    let any_stroke_related = !stroke_keys.is_disjoint(&held)
        || !dash_keys.is_disjoint(&held)
        || !arrow_keys.is_disjoint(&held);

    let mut width = settings.default_stroke_width;

    if any_stroke_related {
        style.insert("stroke".to_string(), "black".to_string());
        style.insert("marker-end".to_string(), "none".to_string());
        style.insert("marker-start".to_string(), "none".to_string());
        style.insert("stroke-dasharray".to_string(), "none".to_string());
        // Reset every time, not just when a dot pattern is applied --
        // otherwise a round cap from an earlier dotted style stays stuck
        // on the object the next time a plain/dashed style is pasted over
        // it (paste-style only overwrites the properties it includes).
        style.insert("stroke-linecap".to_string(), "butt".to_string());
    } else {
        style.insert("stroke".to_string(), "none".to_string());
    }

    for m in &styles.modifiers.strokes {
        if held.contains(m.key.as_str()) {
            let d = m.description.to_lowercase();
            if d.contains("very thick") {
                width = settings.very_thick_width;
            } else if d.contains("thick") {
                width = settings.thick_width;
            }
        }
    }
    style.insert("stroke-width".to_string(), width.to_string());

    let mut has_arrow_end = false;
    let mut has_arrow_start = false;
    for m in &styles.modifiers.arrows {
        if held.contains(m.key.as_str()) {
            has_arrow_end = true;
            if m.description.to_lowercase().contains("start") {
                has_arrow_start = true;
            }
        }
    }
    let mut marker_width = None;
    if has_arrow_end {
        marker_width = Some(width);
        style.insert(
            "marker-end".to_string(),
            format!("url(#marker-arrow-{})", width),
        );
    }
    if has_arrow_start {
        marker_width = Some(width);
        style.insert(
            "marker-start".to_string(),
            format!("url(#marker-arrow-{})", width),
        );
    }

    for m in &styles.modifiers.dashes {
        if held.contains(m.key.as_str()) {
            if m.description.to_lowercase().contains("dot") {
                // Short segments, sized off the actual stroke width so
                // they scale with normal/thick/very-thick, plus a round
                // cap so each segment renders as a filled circle instead
                // of a tiny flat-ended rectangle -- genuine dots.
                style.insert(
                    "stroke-dasharray".to_string(),
                    format!("{},{}", width, 2.0 * settings.pt_multiplier),
                );
                style.insert("stroke-linecap".to_string(), "round".to_string());
            } else {
                // Evenly-sized on/off segments, independent of stroke
                // width, for a proper dashed look.
                style.insert(
                    "stroke-dasharray".to_string(),
                    format!(
                        "{},{}",
                        3.0 * settings.pt_multiplier,
                        2.0 * settings.pt_multiplier
                    ),
                );
            }
        }
    }

    let mut has_fill = false;
    for m in &styles.modifiers.fills {
        if held.contains(m.key.as_str()) {
            has_fill = true;
            let desc = m.description.to_lowercase();
            if desc.contains("white") {
                style.insert("fill".to_string(), "white".to_string());
            } else if desc.contains("light") {
                style.insert("fill".to_string(), "black".to_string());
                style.insert("fill-opacity".to_string(), "0.12".to_string());
            } else {
                style.insert("fill".to_string(), "black".to_string());
            }
            style
                .entry("fill-opacity".to_string())
                .or_insert("1".to_string());
        }
    }
    if has_fill {
        style.insert("marker-end".to_string(), "none".to_string());
        style.insert("marker-start".to_string(), "none".to_string());
    } else {
        style.insert("fill".to_string(), "none".to_string());
        style.insert("fill-opacity".to_string(), "1".to_string());
    }

    ComputedStyle {
        attributes: style,
        marker_width,
    }
}

fn wrap_style_svg(computed: &ComputedStyle) -> String {
    let mut entries: Vec<(&String, &String)> = computed.attributes.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    let style_string = entries
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(";");

    let marker_def = if let Some(w) = computed.marker_width {
        format!(
            r#"<defs id="marker-defs"><marker id="marker-arrow-{w}" orient="auto-start-reverse" refY="0" refX="0" markerHeight="1.690" markerWidth="0.911"><path d="M -1.55415,2.0722 C -1.42464,1.29512 0,0.1295 0.38852,0 0,-0.1295 -1.42464,-1.29512 -1.55415,-2.0722" style="fill:none;stroke:#000000;stroke-width:0.6;stroke-linecap:round;stroke-linejoin:round;stroke-miterlimit:10;stroke-dasharray:none;stroke-opacity:1" /></marker></defs>"#
        )
    } else {
        String::new()
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">{marker_def}<inkscape:clipboard style="{style_string}" /></svg>"#
    )
}
