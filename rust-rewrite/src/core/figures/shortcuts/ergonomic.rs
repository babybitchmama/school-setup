use super::constants::{CONTROL_MASK, XK_DELETE};

/// A single ergonomic action: the real Inkscape keystroke to synthesize
/// in place of whatever we intercepted.
pub struct ErgonomicAction {
    pub keysym: u32,
    pub modifiers: u16,
}

/// Left-handed ergonomic shortcuts, translated into Inkscape's actual
/// default keybindings so the drawing hand (mouse) never has to leave the
/// canvas. Mirrors Castel's `handle_single_key`, but remapped: `c` and `v`
/// take over pencil/bezier here since this project's `w`/`f` are already
/// spoken for by the style-modifier chords in styles.yaml.
pub fn lookup(ch: char, shift_held: bool) -> Option<ErgonomicAction> {
    match (ch, shift_held) {
        ('c', false) => Some(ErgonomicAction { keysym: 'p' as u32, modifiers: 0 }), // Pencil tool
        ('v', false) => Some(ErgonomicAction { keysym: 'b' as u32, modifiers: 0 }), // Bezier/pen tool
        ('x', false) => Some(ErgonomicAction { keysym: '%' as u32, modifiers: 0 }), // Toggle snapping
        ('z', false) => Some(ErgonomicAction { keysym: 'z' as u32, modifiers: CONTROL_MASK }), // Undo
        ('z', true) => Some(ErgonomicAction { keysym: XK_DELETE, modifiers: 0 }), // Delete
        _ => None,
    }
}

/// Whether `ch` is claimed by an ergonomic action at all (for either shift
/// state), used to decide whether a keypress needs to be buffered instead
/// of passed straight through to Inkscape.
pub fn is_ergonomic_key(ch: char) -> bool {
    matches!(ch, 'c' | 'v' | 'x' | 'z')
}
