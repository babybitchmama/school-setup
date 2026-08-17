//! X11 constants the shortcuts engine needs. Kept as plain integers rather
//! than pulling in a keysym-name crate, since the core engine only cares
//! about ASCII letters plus a couple of named modifier keys.

/// Keysym for the left Super/Windows key (ungrabbed so the WM still gets
/// Super+... shortcuts while Inkscape is focused).
pub const XK_SUPER_L: u32 = 0xffeb;
/// Keysym for the left Alt key (same reasoning).
pub const XK_ALT_L: u32 = 0xffe9;

/// Raw X11 KeyButMask bits (stable across the protocol).
pub const SHIFT_MASK: u16 = 1 << 0;
pub const CONTROL_MASK: u16 = 1 << 2;

/// The MIME type Inkscape advertises for its native clipboard format.
/// Pushing an `<inkscape:clipboard>` snippet under this target is what lets
/// Ctrl+Shift+V apply it as a *style* instead of pasting literal SVG markup.
pub const CLIPBOARD_TARGET: &str = "image/x-inkscape-svg";

pub const PID_FILE: &str = "/tmp/lesson-manager/shortcuts.pid";
