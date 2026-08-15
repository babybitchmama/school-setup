use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StylePreset {
    pub name: String,
    pub shortcut: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StyleConfig {
    pub styles: Vec<StylePreset>,
}

pub struct StyleManager {
    presets: HashMap<String, StylePreset>,
}

impl StyleManager {
    pub fn load() -> Self {
        let config_path = shellexpand::tilde("~/.config/lesson-manager/figures/styles.yaml").into_owned();
        let path = PathBuf::from(&config_path);

        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let default_config = r#"styles:"#;
            let _ = fs::write(&path, default_config);
        }

        let presets = if let Ok(content) = fs::read_to_string(&path) {
            let parsed: Result<StyleConfig, _> = serde_yaml::from_str(&content);
            match parsed {
                Ok(cfg) => cfg.styles.into_iter().map(|s| (s.shortcut.clone(), s)).collect(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self { presets }
    }

    pub fn handle_shortcut(&self, shortcut_key: &str) -> bool {
        if let Some(preset) = self.presets.get(shortcut_key) {
            println!("Applying style preset: {}", preset.name);
            self.apply_style_to_selection(&preset.attributes);
            true
        } else {
            false
        }
    }

    fn apply_style_to_selection(&self, attrs: &HashMap<String, String>) {
        let mut sorted: Vec<(&String, &String)> = attrs.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        let style_str = sorted
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        let svg_payload = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg>
  <inkscape:clipboard style="{}" />
</svg>"#,
            style_str
        );

        if let Ok(mut child) = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .arg("-target")
            .arg("image/x-inkscape-svg")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(svg_payload.as_bytes());
            }
            let _ = child.wait();
        }

        // Paste style and instantly escape out of any accidental tool mode
        let _ = Command::new("xdotool")
            .args(["key", "ctrl+shift+v", "Escape"])
            .status();
    }
}
