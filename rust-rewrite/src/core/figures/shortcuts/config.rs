use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub pt_multiplier: f64,
    pub default_stroke_width: f64,
    pub thick_width: f64,
    pub very_thick_width: f64,
    /// Where named custom styles (saved from an actual Inkscape selection
    /// via the "save-style" action) are persisted. Unlike `presets`, this
    /// file is written to at runtime, so it lives separately from
    /// styles.yaml rather than requiring hand-editing.
    pub custom_styles_path: String,
    #[serde(default)]
    pub terminal_class_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub shortcut: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Preset {
    pub name: String,
    pub shortcut: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModifierKey {
    pub key: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Modifiers {
    #[serde(default)]
    pub strokes: Vec<ModifierKey>,
    #[serde(default)]
    pub arrows: Vec<ModifierKey>,
    #[serde(default)]
    pub dashes: Vec<ModifierKey>,
    #[serde(default)]
    pub fills: Vec<ModifierKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemBinding {
    pub key: String,
    pub item: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

/// A single key that expands to a whole set of modifier keys, as if they
/// were all held simultaneously. Unlike `Preset` (a fixed attribute map
/// snapshot), a combo is resolved through the exact same procedural
/// computation as a real chord -- so it stays correct if `settings` (pt
/// multiplier, stroke widths, ...) ever changes, instead of drifting out
/// of sync with a hand-written attribute map.
///
/// Expansion is one level deep only: a combo's `keys` must name actual
/// stroke/dash/arrow/fill modifier keys, not other combos.
#[derive(Debug, Deserialize, Clone)]
pub struct Combo {
    pub key: String,
    pub keys: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextConfig {
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
}

fn default_font() -> String {
    "sans-serif".to_string()
}

fn default_font_size() -> f64 {
    14.0
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font: default_font(),
            font_size: default_font_size(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StylesConfig {
    pub settings: Settings,
    #[serde(default)]
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub modifiers: Modifiers,
    #[serde(default)]
    pub text_config: TextConfig,
    #[serde(default)]
    pub items: Vec<ItemBinding>,
    #[serde(default)]
    pub combos: Vec<Combo>,
}

impl StylesConfig {
    pub fn load() -> Self {
        let path = shellexpand::tilde("~/.config/lesson-manager/figures/styles.yaml").into_owned();
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
        let styles: StylesConfig =
            serde_yaml::from_str(&contents).expect("Failed to parse styles.yaml");
        styles.warn_about_collisions();
        styles
    }

    /// A single keypress is looked up against `presets` first, then
    /// `combos`; the `modifiers` sections only combine into a style when
    /// 2+ keys are held at once (or one of those keys is a combo that
    /// expands to several). Since these sections can reuse the same
    /// letter for different meanings, warn loudly so any collision is a
    /// deliberate choice.
    ///
    /// This doesn't check `items` bindings for collisions yet -- e.g. your
    /// current styles.yaml binds 'b' both as the "blue-stroke" preset and
    /// as the "snap" item. Manager resolves that by checking actions, then
    /// items, then presets, in that order (items win over presets on a
    /// single keypress) -- worth knowing about, not fixed automatically.
    fn warn_about_collisions(&self) {
        let mut modifier_keys: HashMap<&str, &str> = HashMap::new();
        for (group_name, group) in [
            ("strokes", &self.modifiers.strokes),
            ("arrows", &self.modifiers.arrows),
            ("dashes", &self.modifiers.dashes),
            ("fills", &self.modifiers.fills),
        ] {
            for m in group {
                modifier_keys.insert(m.key.as_str(), group_name);
            }
        }
        for preset in &self.presets {
            if let Some(group_name) = modifier_keys.get(preset.shortcut.as_str()) {
                println!(
                    "Warning: styles.yaml preset '{}' uses shortcut '{}', which is also \
                     a modifier key in '{}'. On a single keypress the preset wins; the \
                     modifier meaning is only reachable inside a multi-key chord.",
                    preset.name, preset.shortcut, group_name
                );
            }
        }
        for combo in &self.combos {
            if let Some(group_name) = modifier_keys.get(combo.key.as_str()) {
                println!(
                    "Warning: styles.yaml combo '{}' uses key '{}', which is also a \
                     modifier key in '{}'. On a single keypress the combo wins, so '{}' \
                     alone can no longer be used for its plain modifier meaning.",
                    combo.key, combo.key, group_name, combo.key
                );
            }
            if self.presets.iter().any(|p| p.shortcut == combo.key) {
                println!(
                    "Warning: styles.yaml combo '{}' uses the same key as a preset. \
                     The preset wins on a single keypress.",
                    combo.key
                );
            }
            for k in &combo.keys {
                if !modifier_keys.contains_key(k.as_str()) {
                    println!(
                        "Warning: styles.yaml combo '{}' references '{}', which isn't a \
                         stroke/dash/arrow/fill modifier key. It will be ignored when the \
                         combo is expanded.",
                        combo.key, k
                    );
                }
            }
        }
    }

    pub fn find_preset(&self, key: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.shortcut == key)
    }

    pub fn find_action(&self, key: &str) -> Option<&Action> {
        self.actions
            .iter()
            .find(|a| a.shortcut.to_lowercase() == key)
    }

    pub fn find_combo(&self, key: &str) -> Option<&Combo> {
        self.combos.iter().find(|c| c.key == key)
    }
}
