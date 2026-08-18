use std::collections::HashMap;

use super::config::StylesConfig;
use super::constants::{CONTROL_MASK, SHIFT_MASK, XK_DELETE};

/// The concrete Inkscape keystroke behind a named `item`.
#[derive(Debug, Clone, Copy)]
pub struct ItemAction {
    pub keysym: u32,
    pub modifiers: u16,
}

fn tap(keysym: char) -> ItemAction {
    ItemAction {
        keysym: keysym as u32,
        modifiers: 0,
    }
}

fn tap_with(keysym: char, modifiers: u16) -> ItemAction {
    ItemAction {
        keysym: keysym as u32,
        modifiers,
    }
}

/// The fixed catalog of actions this engine knows how to perform, keyed by
/// name so styles.yaml's `items:` list can bind any key to any of these.
/// Tool letters match Inkscape's own default keybindings (Edit > Preferences
/// > Interface > Keyboard has the authoritative list for a given install,
/// in case a distro or a customized keymap differs from these defaults).
fn resolve_item(name: &str) -> Option<ItemAction> {
    match name {
        // Selection / editing
        "selector" | "select" => Some(tap('s')),
        "node" | "node-editor" => Some(tap('n')),

        // Shape tools
        "rectangle" | "rect" => Some(tap('r')),
        "ellipse" | "circle" | "arc" => Some(tap('e')),
        "star" => Some(tap('*')),
        "spiral" => Some(tap('i')),
        "3dbox" | "box" | "3d-box" => Some(tap('x')),

        // Freehand drawing
        "pencil" | "freehand" => Some(tap('p')),
        "bezier" | "pen" => Some(tap('b')),
        "calligraphy" => Some(tap('c')),

        // Paint / color
        "gradient" => Some(tap('g')),
        "dropper" | "eyedropper" => Some(tap('d')),
        "paintbucket" | "bucket" | "paint-bucket" => Some(tap('u')),

        // Manipulation
        "tweak" => Some(tap('w')),
        "spray" => Some(tap_with('w', SHIFT_MASK)),
        "eraser" => Some(tap_with('e', SHIFT_MASK)),
        "connector" => Some(tap('o')),

        // Text / view
        "text" => Some(tap('t')),
        "zoom" => Some(tap('z')),
        "measure" => Some(tap_with('m', SHIFT_MASK)),

        // Canvas / document actions (not tool switches)
        "snap" => Some(tap('%')),
        "undo" => Some(tap_with('z', CONTROL_MASK)),
        "redo" => Some(tap_with('y', CONTROL_MASK)),
        "delete" => Some(ItemAction {
            keysym: XK_DELETE,
            modifiers: 0,
        }),

        _ => None,
    }
}

fn parse_modifiers(names: &[String]) -> u16 {
    names.iter().fold(0u16, |acc, name| {
        acc | match name.to_lowercase().as_str() {
            "shift" => SHIFT_MASK,
            "ctrl" | "control" => CONTROL_MASK,
            other => {
                println!(
                    "Warning: styles.yaml items entry has unknown modifier '{}'",
                    other
                );
                0
            }
        }
    })
}

/// The fully resolved (trigger key, trigger modifiers) -> action table,
/// built once from styles.yaml's `items:` list.
pub struct Bindings(HashMap<(char, u16), ItemAction>);

impl Bindings {
    pub fn from_config(styles: &StylesConfig) -> Self {
        let mut map = HashMap::new();

        for binding in &styles.items {
            let Some(ch) = binding.key.chars().next() else {
                continue;
            };
            let Some(action) = resolve_item(&binding.item) else {
                println!(
                    "Warning: styles.yaml items entry '{}' -> '{}' names an unknown item; skipping.",
                    binding.key, binding.item
                );
                continue;
            };
            let trigger_modifiers = parse_modifiers(&binding.modifiers);

            if map.insert((ch, trigger_modifiers), action).is_some() {
                println!(
                    "Warning: styles.yaml has more than one items entry for key '{}' with the \
                     same modifiers; the last one in the file wins.",
                    binding.key
                );
            }
        }

        Bindings(map)
    }

    pub fn lookup(&self, ch: char, modifiers_held: u16) -> Option<ItemAction> {
        self.0.get(&(ch, modifiers_held)).copied()
    }

    pub fn keys(&self) -> impl Iterator<Item = (char, u16)> + '_ {
        self.0.keys().copied()
    }
}
