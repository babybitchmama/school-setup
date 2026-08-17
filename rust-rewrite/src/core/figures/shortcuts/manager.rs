use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    self, Allow, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, GrabMode, KeyPressEvent,
    KeyReleaseEvent, ModMask, Window,
};
use x11rb::rust_connection::RustConnection;

use crate::config::LessonManagerConfigFile;

use super::clipboard;
use super::config::StylesConfig;
use super::constants::{CONTROL_MASK, SHIFT_MASK, XK_ALT_L, XK_SUPER_L};
use super::normal;

/// Owns one X11 connection dedicated to a single Inkscape window. Mirrors
/// Castel's `Manager` class in `main.py`: each watched window gets its own
/// connection, its own key grab, and runs on its own thread.
pub struct Manager {
    conn: RustConnection,
    window: Window,
    min_keycode: u8,
    keysyms_per_keycode: u8,
    keysyms: Vec<u32>,
    config: LessonManagerConfigFile,
}

impl Manager {
        pub fn new(window: Window, config: LessonManagerConfigFile) -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, _screen_num) = x11rb::connect(None)?;
        let setup = conn.setup();
        let min_keycode = setup.min_keycode;
        let max_keycode = setup.max_keycode;
        let count = max_keycode - min_keycode + 1;

        let mapping = conn.get_keyboard_mapping(min_keycode, count)?.reply()?;

        Ok(Manager {
            conn,
            window,
            min_keycode,
            keysyms_per_keycode: mapping.keysyms_per_keycode,
            keysyms: mapping.keysyms,
            config,
        })
    }

    fn keysym_for_keycode(&self, keycode: u8) -> Option<u32> {
        if keycode < self.min_keycode {
            return None;
        }
        let row = (keycode - self.min_keycode) as usize * self.keysyms_per_keycode as usize;
        self.keysyms.get(row).copied().filter(|&ks| ks != 0)
    }

    fn keycode_for_keysym(&self, keysym: u32) -> Option<u8> {
        let per = self.keysyms_per_keycode as usize;
        for (i, chunk) in self.keysyms.chunks(per).enumerate() {
            if chunk.first().copied() == Some(keysym) {
                return Some(self.min_keycode + i as u8);
            }
        }
        None
    }

    /// ASCII/Latin-1 keysyms share their codepoint with the character they
    /// represent, which covers everything our chord engine needs (letters,
    /// digits, punctuation). Named keys fall outside this range and are
    /// ignored for now.
    fn char_for_keysym(keysym: u32) -> Option<char> {
        if (0x20..=0x7e).contains(&keysym) {
            char::from_u32(keysym)
        } else {
            None
        }
    }

    /// Grabs every key sent to this window so we see it before Inkscape
    /// does, then releases the window-manager keys (Super, Alt) so things
    /// like Alt+Tab keep working while Inkscape is focused.
    pub fn grab(&self) -> Result<(), Box<dyn std::error::Error>> {
        xproto::grab_key(
            &self.conn,
            true,
            self.window,
            ModMask::ANY,
            0, // AnyKey
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?
        .check()?;

        for keysym in [XK_SUPER_L, XK_ALT_L] {
            if let Some(keycode) = self.keycode_for_keysym(keysym) {
                xproto::ungrab_key(&self.conn, keycode, self.window, ModMask::ANY)?.check()?;
            }
        }

        self.conn
            .change_window_attributes(
                self.window,
                &ChangeWindowAttributesAux::new().event_mask(
                    EventMask::KEY_PRESS | EventMask::KEY_RELEASE | EventMask::STRUCTURE_NOTIFY,
                ),
            )?
            .check()?;

        self.conn.flush()?;
        Ok(())
    }

    pub fn ungrab(&self) -> Result<(), Box<dyn std::error::Error>> {
        xproto::ungrab_key(&self.conn, 0, self.window, ModMask::ANY)?.check()?;
        self.conn.flush()?;
        Ok(())
    }

    /// Synthesizes a KeyPress+KeyRelease pair sent directly to the Inkscape
    /// window, used to trigger Ctrl+Shift+V after loading a style onto the
    /// clipboard.
    fn press(&self, ch: char, modifiers: u16) -> Result<(), Box<dyn std::error::Error>> {
        let Some(keycode) = self.keycode_for_keysym(ch as u32) else {
            return Ok(());
        };

        let press_event = KeyPressEvent {
            response_type: xproto::KEY_PRESS_EVENT,
            detail: keycode,
            sequence: 0,
            time: x11rb::CURRENT_TIME,
            root: self.window,
            event: self.window,
            child: 0,
            root_x: 0,
            root_y: 0,
            event_x: 0,
            event_y: 0,
            state: modifiers.into(),
            same_screen: false,
        };
        self.conn
            .send_event(true, self.window, EventMask::NO_EVENT, press_event)?;

        let release_event = KeyReleaseEvent {
            response_type: xproto::KEY_RELEASE_EVENT,
            ..press_event
        };
        self.conn
            .send_event(true, self.window, EventMask::NO_EVENT, release_event)?;

        self.conn.flush()?;
        Ok(())
    }

    /// The core event loop. Buffers held keys; on release, resolves them to
    /// a style via `normal::build_style_svg`, pushes it to the clipboard,
    /// and pastes it into Inkscape via a synthesized Ctrl+Shift+V.
    pub fn listen(&self, styles: &StylesConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.grab()?;

        let mut pressed: Vec<String> = Vec::new();

        loop {
            let event = self.conn.wait_for_event()?;

            match event {
                Event::KeyPress(ev) => {
                    if let Some(keysym) = self.keysym_for_keycode(ev.detail) {
                        if let Some(ch) = Self::char_for_keysym(keysym) {
                            pressed.push(ch.to_string());
                        }
                    }
                    self.conn
                        .allow_events(Allow::REPLAY_KEYBOARD, x11rb::CURRENT_TIME)?;
                    self.conn.flush()?;
                }
                Event::KeyRelease(_) => {
                    self.conn
                        .allow_events(Allow::REPLAY_KEYBOARD, x11rb::CURRENT_TIME)?;
                    self.conn.flush()?;

                    let key_str = pressed.join("+");

                    if let Some(action) = styles.find_action(&key_str) {
                        match action.name.as_str() {
                            "neovim-latex-code" => {
                                println!("Opening Neovim for raw LaTeX code...");
                                let math_mgr = crate::core::figures::shortcuts::math::MathMacroManager::new();

                                let terminal = &self.config.terminal; // e.g. "alacritty"
                                let editor = &self.config.editor;         // e.g. "nvim"
                                let editor_mode = &self.config.inkscape_mode;   // e.g. "floating"

                                match math_mgr.edit_and_compile(false, terminal, editor, editor_mode, styles) {
                                    Ok(_svg) => {
                                        if let Err(e) = self.press('v', CONTROL_MASK) {
                                            println!("Failed to send paste event to Inkscape window: {}", e);
                                        }
                                    }
                                    Err(e) => println!("Math macro error: {}", e),
                                }
                            }
                            "neovim-latex-compiled" => {
                                println!("Opening Neovim, compiling LaTeX to SVG...");
                                let math_mgr = crate::core::figures::shortcuts::math::MathMacroManager::new();

                                let terminal = &self.config.terminal;
                                let editor = &self.config.editor;
                                let editor_mode = &self.config.inkscape_mode;

                                match math_mgr.edit_and_compile(true, terminal, editor, editor_mode, styles) {
                                    Ok(_svg) => {
                                        // Payload is already copied to clipboard inside edit_and_compile
                                        if let Err(e) = self.press('v', CONTROL_MASK) {
                                            println!("Failed to send paste event to Inkscape window: {}", e);
                                        }
                                    }
                                    Err(e) => println!("Math macro error: {}", e),
                                }
                            }
                            _ => {}
                        }
                    } else if let Some(svg) = normal::build_style_svg(styles, &pressed) {
                        if let Err(e) = clipboard::copy(&svg) {
                            println!("Failed to copy style to clipboard: {}", e);
                        } else {
                            let _ = self.press('v', CONTROL_MASK | SHIFT_MASK);
                        }
                    }

                    pressed.clear();
                }
                Event::DestroyNotify(ev) => {
                    if ev.window == self.window {
                        self.ungrab()?;
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }
}
