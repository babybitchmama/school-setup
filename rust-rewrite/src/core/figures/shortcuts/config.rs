use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub pt_multiplier: f64,
    pub default_stroke_width: f64,
    pub thick_width: f64,
    pub very_thick_width: f64,
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

    /// A single keypress is looked up against `presets` first; the
    /// `modifiers` sections only combine into a style when 2+ keys are held
    /// at once. Since both sections can reuse the same letter for different
    /// meanings, warn loudly so any collision is a deliberate choice.
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
    }

    pub fn find_preset(&self, key: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.shortcut == key)
    }

    pub fn find_action(&self, key: &str) -> Option<&Action> {
        self.actions.iter().find(|a| a.shortcut == key)
    }

    /// Whether this key means something to us at all: a preset shortcut, a
    /// modifier-chord letter, or an ergonomic action. Anything else gets
    /// passed straight through to Inkscape untouched.
    pub fn is_relevant_key(&self, ch: char) -> bool {
        let s = ch.to_string();
        self.find_preset(&s).is_some()
        || self.modifiers.strokes.iter().any(|m| m.key == s)
        || self.modifiers.arrows.iter().any(|m| m.key == s)
        || self.modifiers.dashes.iter().any(|m| m.key == s)
        || self.modifiers.fills.iter().any(|m| m.key == s)
        || super::ergonomic::is_ergonomic_key(ch)
    }
}
