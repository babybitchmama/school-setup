use std::sync::Arc;

use crate::config::LessonManagerConfigFile;
use crate::core::figures::shortcuts::custom_objects::object_path;
use clap::Command;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    self, Allow, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, GrabMode,
    KeyPressEvent, KeyReleaseEvent, ModMask, Window,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt;

use super::clipboard;
use super::config::StylesConfig;
use super::constants::{CONTROL_MASK, SHIFT_MASK, XK_ALT_L, XK_DELETE, XK_SUPER_L};
use super::ergonomic::Bindings;
use super::normal;

/// Spawns `inkscape <path>`, waits for its window to appear and relabels
/// it with `class_name`, then blocks until the user closes it -- same
/// blocking contract the old `action_edit_style` had via `.status()`.
/// If relabeling fails for any reason, Inkscape still opens normally with
/// its default class; a missing WM rule match is far less disruptive than
/// the whole edit flow silently doing nothing.
fn open_floating_inkscape(path: &Path) {
    // 1. Tell bspwm to make the next incoming Inkscape window float.
    let _ = std::process::Command::new("bspc")
        .args(["rule", "-a", "Inkscape", "-o", "state=floating"])
        .output();

    // 2. Explicitly use std::process::Command to spawn Inkscape
    let mut child = match std::process::Command::new("inkscape")
        .arg("--with-gui")
        .arg(path)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to launch Inkscape: {}", e);
            return;
        }
    };

    let _ = child.wait();
}

/// Opens its own short-lived X11 connection, watches the root window for
/// CreateNotify events, and matches the new window to `target_pid` via
/// `_NET_WM_PID` -- more precise than matching on WM_CLASS content (as
/// the daemon's own Inkscape-window watcher does), since we're about to
/// change that property anyway and don't want to race a second Inkscape
/// window someone might have open. Gives up after 10s.
fn relabel_next_inkscape_window(
    target_pid: u32,
    class_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::SUBSTRUCTURE_NOTIFY),
    )?
    .check()?;

    let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut candidates: Vec<Window> = Vec::new();

    while Instant::now() < deadline {
        if let Some(Event::CreateNotify(ev)) = conn.poll_for_event()? {
            candidates.push(ev.window);
        }

        // _NET_WM_PID may not be set the instant CreateNotify fires, so we
        // keep re-checking every candidate window each pass rather than
        // only checking it once.
        if let Some(pos) = candidates
            .iter()
            .position(|&w| get_window_pid(&conn, w, net_wm_pid) == Some(target_pid))
        {
            set_wm_class(&conn, candidates[pos], class_name)?;
            conn.flush()?;
            return Ok(());
        }

        thread::sleep(Duration::from_millis(20));
    }

    Err("timed out waiting for the new Inkscape window to appear".into())
}

fn get_window_pid(conn: &impl Connection, window: Window, net_wm_pid: xproto::Atom) -> Option<u32> {
    let cookie = conn
        .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()?;
    cookie.reply().ok()?.value32()?.next()
}

fn set_wm_class(
    conn: &impl Connection,
    window: Window,
    class_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // WM_CLASS is two null-terminated strings back to back: instance, then
    // class. Setting both to the same value mirrors how kitty's --class
    // behaves, so a WM rule can match on either.
    let mut value = class_name.as_bytes().to_vec();
    value.push(0);
    value.extend_from_slice(class_name.as_bytes());
    value.push(0);

    conn.change_property8(
        xproto::PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        &value,
    )?
    .check()?;
    Ok(())
}

/// Owns one X11 connection dedicated to a single Inkscape window. Mirrors
/// Castel's `Manager` class in `main.py`: each watched window gets its own
/// connection, its own key grab, and runs on its own thread.
pub struct Manager {
    conn: RustConnection,
    window: Window,
    min_keycode: u8,
    keysyms_per_keycode: u8,
    keysyms: Vec<u32>,
    config: Arc<LessonManagerConfigFile>,
}

impl Manager {
    pub fn new(
        window: Window,
        config: Arc<LessonManagerConfigFile>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

    /// Finds a keycode/level pair that produces `keysym`, searching every
    /// shift level (not just level 0). This matters for keys like '%',
    /// which on a standard layout only exists as the *shifted* form of the
    /// '5' key -- looking at level 0 only would never find it.
    fn locate_keysym(&self, keysym: u32) -> Option<(u8, usize)> {
        let per = self.keysyms_per_keycode as usize;
        if per == 0 {
            return None;
        }
        for (i, chunk) in self.keysyms.chunks(per).enumerate() {
            if let Some(level) = chunk.iter().position(|&ks| ks == keysym) {
                return Some((self.min_keycode + i as u8, level));
            }
        }
        None
    }

    fn keycode_for_keysym(&self, keysym: u32) -> Option<u8> {
        self.locate_keysym(keysym).map(|(keycode, _)| keycode)
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
    ///
    /// Note the keyboard grab mode is `Async`, not `Sync`. That means a
    /// grabbed key is *never* frozen and is delivered to us only -- it is
    /// not queued up waiting for a decision. Consequently
    /// `allow_events(REPLAY_KEYBOARD)` (used throughout `listen`) is a
    /// no-op here: there's nothing frozen to replay, so it does not forward
    /// the key to Inkscape. Any key we want Inkscape to actually see has to
    /// be manually resynthesized via `press`/`passthrough`.
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

    fn action_apply_custom_style(&self) {
        let names = super::custom_styles::list_style_names();
        if names.is_empty() {
            println!(
                "No custom styles found in {}",
                super::custom_styles::styles_dir().display()
            );
            return;
        }

        let Some(style_name) = crate::rofi::select::select_from_rofi(
            names,
            &self.config.rofi_options,
            "Apply Style".to_string(),
        ) else {
            return;
        };

        // Grab the current selection's real markup via a genuine copy, so
        // we can tell what type of object is selected. The sleep gives the
        // X clipboard a moment to actually update before we read it back.
        if let Err(e) = self.press('c' as u32, CONTROL_MASK) {
            println!("Failed to copy selection for style lookup: {}", e);
            return;
        }
        thread::sleep(Duration::from_millis(150));

        let copied = match super::clipboard::get() {
            Ok(content) => content,
            Err(e) => {
                println!("Failed to read clipboard: {}", e);
                return;
            }
        };

        match super::custom_styles::apply::resolve_style_svg(&style_name, &copied) {
            Ok(style_svg) => {
                if let Err(e) = super::clipboard::copy(&style_svg) {
                    println!("Failed to copy resolved style to clipboard: {}", e);
                    return;
                }
                if let Err(e) = self.press('v' as u32, CONTROL_MASK | SHIFT_MASK) {
                    println!("Failed to paste style: {}", e);
                }
            }
            Err(msg) => println!("Cannot apply style '{}': {}", style_name, msg),
        }
    }

    fn action_create_custom_style(&self) {
        let Some(raw_name) =
            crate::rofi::input::prompt_input("New Style Name", &self.config.rofi_options)
        else {
            return;
        };
        let name = raw_name.trim().to_lowercase().replace(' ', "-");
        if name.is_empty() {
            return;
        }

        if super::custom_styles::style_path(&name).exists() {
            let confirm = crate::rofi::select::select_from_rofi(
                vec!["y".to_string(), "n".to_string()],
                &self.config.rofi_options,
                format!("'{}' exists -- merge new shape(s) into it?", name),
            );
            if confirm.as_deref() != Some("y") {
                return;
            }
        }

        let mode = crate::rofi::select::select_from_rofi(
            vec![
                "From current selection".to_string(),
                "Start blank in Inkscape".to_string(),
            ],
            &self.config.rofi_options,
            "Save Style".to_string(),
        );

        match mode.as_deref() {
            Some("From current selection") => {
                if let Err(e) = self.press('c' as u32, CONTROL_MASK) {
                    println!("Failed to copy selection: {}", e);
                    return;
                }
                thread::sleep(Duration::from_millis(150));

                match super::clipboard::get() {
                    Ok(content) if !content.trim().is_empty() => {
                        match super::custom_styles::save::merge_into_style(&name, &content) {
                            Ok(path) => println!("Saved style '{}' to {}", name, path.display()),
                            Err(e) => println!("Failed to save style: {}", e),
                        }
                    }
                    Ok(_) => println!("Nothing was selected to copy."),
                    Err(e) => println!("Failed to read clipboard: {}", e),
                }
            }
            Some("Start blank in Inkscape") => {
                let scratch_path =
                    PathBuf::from(format!("/tmp/lesson-manager/styles/{}.svg", name));

                if let Err(e) = super::custom_styles::save::create_blank_scratch_svg(&scratch_path)
                {
                    println!("Failed to create scratch file: {}", e);
                    return;
                }

                match std::process::Command::new("inkscape")
                    .arg(&scratch_path)
                    .status()
                {
                    Ok(_) => {
                        match super::custom_styles::save::promote_scratch_to_style(
                            &name,
                            &scratch_path,
                        ) {
                            Ok(path) => println!("Saved style '{}' to {}", name, path.display()),
                            Err(e) => println!("Failed to save style: {}", e),
                        }
                    }
                    Err(e) => println!("Failed to launch Inkscape: {}", e),
                }
            }
            _ => {}
        }
    }

    fn action_edit_custom_style(&self) {
        let names = super::custom_styles::list_style_names();
        if names.is_empty() {
            println!(
                "No custom styles found in {}",
                super::custom_styles::styles_dir().display()
            );
            return;
        }

        let Some(style_name) = crate::rofi::select::select_from_rofi(
            names,
            &self.config.rofi_options,
            "Edit Style".to_string(),
        ) else {
            return;
        };

        let path = super::custom_styles::style_path(&style_name);

        if let Err(e) = std::process::Command::new("inkscape").arg(&path).status() {
            println!(
                "Failed to launch Inkscape for style '{}': {}",
                style_name, e
            );
        }
        // No live re-parse on save yet, per your #5 -- apply/list just
        // read the file fresh from disk each time, so the edit takes
        // effect the next time you press `n`.
    }

    /// Synthesizes a KeyPress+KeyRelease pair sent directly to the Inkscape
    /// window, used both to relay item/tool shortcuts and to trigger
    /// Ctrl+Shift+V after loading a style onto the clipboard. `keysym` is
    /// looked up across all shift levels; if it's only reachable at level 1
    /// (e.g. '%' on a standard layout), Shift is added automatically on
    /// top of whatever `modifiers` already asks for.
    fn press(&self, keysym: u32, modifiers: u16) -> Result<(), Box<dyn std::error::Error>> {
        let Some((keycode, level)) = self.locate_keysym(keysym) else {
            return Ok(());
        };

        let state = if level == 1 {
            modifiers | SHIFT_MASK
        } else {
            modifiers
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
            state: state.into(),
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

    /// Relays raw `(keysym, modifier-state)` pairs to Inkscape exactly as
    /// they were physically pressed. This is the fallback path for any key
    /// our grab intercepted but that doesn't resolve to a shortcut -- see
    /// the note on `grab` for why manual resynthesis, rather than
    /// `allow_events`, is what actually gets a key to Inkscape.
    fn passthrough(&self, raw: &[(u32, u16)]) {
        for &(keysym, state) in raw {
            if let Err(e) = self.press(keysym, state) {
                println!(
                    "Failed to relay key (keysym {:#06x}) to Inkscape: {}",
                    keysym, e
                );
            }
        }
    }

    /// The core event loop. Buffers held keys (both as chars, for chord/
    /// action lookups, and as raw keysym+state pairs, so unresolved keys
    /// can still be relayed) and resolves them on release.
    ///
    /// Shortcut handling can be toggled off via the `toggle-daemon` action
    /// (bound to `` ` `` in styles.yaml). While inactive, every key is
    /// relayed to Inkscape untouched, with no chord/style/action
    /// resolution at all -- only the toggle key itself is still watched,
    /// so there's always a way back in. While active, named actions are
    /// checked first, then item/tool bindings, then style chords; a key
    /// that matches none of those is relayed via `passthrough` so ordinary
    /// typing, Escape, Backspace, arrow keys, etc. keep working normally.
    pub fn listen(&self, styles: &StylesConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.grab()?;

        let bindings = Bindings::from_config(styles);
        let mut active = true;
        let mut pressed: Vec<String> = Vec::new();
        let mut pressed_raw: Vec<(u32, u16)> = Vec::new();

        loop {
            let event = self.conn.wait_for_event()?;

            match event {
                Event::KeyPress(ev) => {
                    if let Some(keysym) = self.keysym_for_keycode(ev.detail) {
                        let state: u16 = ev.state.into();

                        // Recorded for every key, including ones outside
                        // char_for_keysym's ASCII range (Escape, Backspace,
                        // arrows, ...), so passthrough can still relay them.
                        let raw = (keysym, state);
                        if !pressed_raw.contains(&raw) {
                            pressed_raw.push(raw);
                        }

                        if let Some(ch) = Self::char_for_keysym(keysym) {
                            // Guard against auto-repeat: holding a key down
                            // sends repeated KeyPress events, which would
                            // otherwise inflate `pressed` past 1 entry and
                            // break single-key preset/item lookups.
                            let s = ch.to_string();
                            if !pressed.contains(&s) {
                                pressed.push(s);
                            }
                        }
                    }
                    self.conn
                        .allow_events(Allow::REPLAY_KEYBOARD, x11rb::CURRENT_TIME)?;
                    self.conn.flush()?;
                }
                Event::KeyRelease(ev) => {
                    self.conn
                        .allow_events(Allow::REPLAY_KEYBOARD, x11rb::CURRENT_TIME)?;
                    self.conn.flush()?;

                    let state: u16 = ev.state.into();
                    let shift_held = state & SHIFT_MASK != 0;
                    let control_held = state & CONTROL_MASK != 0;

                    // Named actions ("t", "Shift+t", ...) need the
                    // modifier state folded into the lookup key, since the
                    // held-char list alone can't distinguish "t" from
                    // "Shift+t" (Shift itself never appears as a char).
                    let mut action_key = String::new();
                    if control_held {
                        action_key.push_str("ctrl+");
                    }
                    if shift_held {
                        action_key.push_str("shift+");
                    }
                    action_key.push_str(&pressed.join("+").to_lowercase());

                    let matched_action = styles.find_action(&action_key);
                    let is_toggle = matched_action.is_some_and(|a| a.name == "toggle-daemon");

                    if is_toggle {
                        active = !active;
                        println!(
                            "Inkscape shortcuts {}",
                            if active { "enabled" } else { "disabled" }
                        );
                    } else if !active || control_held {
                        // Shortcuts are off: don't resolve anything else,
                        // just relay what was held.
                        self.passthrough(&pressed_raw);
                    } else if let Some(action) = matched_action {
                        self.run_action(action, styles);
                    } else {
                        let handled_by_item = pressed.len() == 1 && {
                            let ch = pressed[0].chars().next().unwrap();
                            let modifiers_held = (if control_held { CONTROL_MASK } else { 0 })
                                | (if shift_held { SHIFT_MASK } else { 0 });

                            match bindings.lookup(ch, modifiers_held) {
                                Some(item_action) => {
                                    if let Err(e) =
                                        self.press(item_action.keysym, item_action.modifiers)
                                    {
                                        println!(
                                            "Failed to send tool shortcut for '{}': {}",
                                            ch, e
                                        );
                                    }
                                    true
                                }
                                None => false,
                            }
                        };

                        if !handled_by_item {
                            if let Some(svg) = normal::build_style_svg(styles, &pressed) {
                                if let Err(e) = clipboard::copy(&svg) {
                                    println!("Failed to copy style to clipboard: {}", e);
                                } else {
                                    let _ = self.press('v' as u32, CONTROL_MASK | SHIFT_MASK);
                                }
                            } else {
                                // Nothing claimed these keys: relay them so
                                // ordinary typing/navigation still reaches
                                // Inkscape instead of being swallowed.
                                self.passthrough(&pressed_raw);
                            }
                        }
                    }

                    pressed.clear();
                    pressed_raw.clear();
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

    fn action_apply_custom_object(&self) {
        let names = super::custom_objects::list_object_names();
        if names.is_empty() {
            println!(
                "No custom objects found in {}",
                super::custom_objects::objects_dir().display()
            );
            return;
        }

        let Some(object_name) = crate::rofi::select::select_from_rofi(
            names,
            &self.config.rofi_options,
            "Paste Object".to_string(),
        ) else {
            return;
        };

        let svg_content = match super::custom_objects::read_object_svg(&object_name) {
            Ok(content) => content,
            Err(e) => {
                println!("Failed to read object '{}': {}", object_name, e);
                return;
            }
        };

        if let Err(e) = super::clipboard::copy(&svg_content) {
            println!(
                "Failed to copy object '{}' to clipboard: {}",
                object_name, e
            );
            return;
        }

        if let Err(e) = self.press('v' as u32, CONTROL_MASK) {
            println!("Failed to paste object '{}': {}", object_name, e);
        }
    }

    fn action_create_custom_object(&self) {
        let names = super::custom_styles::list_style_names();
        if names.is_empty() {
            println!(
                "No custom styles found in {}",
                super::custom_styles::styles_dir().display()
            );
            return;
        }

        let Some(style_name) = crate::rofi::select::select_from_rofi(
            names,
            &self.config.rofi_options,
            "Edit Style".to_string(),
        ) else {
            return;
        };

        let path = super::custom_styles::style_path(&style_name);
        open_floating_inkscape(&path);
        // No live re-parse on save yet -- apply/list just read the file fresh
        // from disk each time, so the edit takes effect the next time you
        // press the apply-style key.
    }

    fn action_edit_custom_object(&self) {
        let names = super::custom_objects::list_object_names();
        if names.is_empty() {
            println!(
                "No custom objects found in {}",
                super::custom_objects::objects_dir().display()
            );
            return;
        }

        let Some(object_name) = crate::rofi::select::select_from_rofi(
            names,
            &self.config.rofi_options,
            "Edit Object".to_string(),
        ) else {
            return;
        };

        let path = super::custom_objects::object_path(&object_name);
        open_floating_inkscape(&path);
    }

    fn run_action(&self, action: &super::config::Action, styles: &StylesConfig) {
        match action.name.as_str() {
            "neovim-latex-code" => {
                println!("Opening Neovim for raw LaTeX code...");
                let math_mgr = super::math::MathMacroManager::new();

                let terminal = &self.config.terminal;
                let editor = &self.config.editor;
                let inkscape_mode = &self.config.inkscape_mode;

                match math_mgr.edit_and_compile(
                    false,
                    terminal,
                    editor,
                    inkscape_mode,
                    styles,
                    Some(self.config.terminal_class_name.clone()),
                ) {
                    Ok(_svg) => {
                        if let Err(e) = self.press('v' as u32, CONTROL_MASK) {
                            println!("Failed to send paste event to Inkscape window: {}", e);
                        }
                    }
                    Err(e) => println!("Math macro error: {}", e),
                }
            }
            "neovim-latex-compiled" => {
                println!("Opening Neovim, compiling LaTeX to SVG...");
                let math_mgr = super::math::MathMacroManager::new();

                let terminal = &self.config.terminal;
                let editor = &self.config.editor;
                let inkscape_mode = &self.config.inkscape_mode;

                match math_mgr.edit_and_compile(
                    true,
                    terminal,
                    editor,
                    inkscape_mode,
                    styles,
                    Some(self.config.terminal_class_name.clone()),
                ) {
                    Ok(_svg) => {
                        // Payload is already copied to clipboard inside edit_and_compile
                        if let Err(e) = self.press('v' as u32, CONTROL_MASK) {
                            println!("Failed to send paste event to Inkscape window: {}", e);
                        }
                    }
                    Err(e) => println!("Math macro error: {}", e),
                }
            }
            "apply-custom-style" => {
                self.action_apply_custom_style();
            }
            "create-custom-style" => {
                self.action_create_custom_style();
            }
            "edit-custom-style" => {
                self.action_edit_custom_style();
            }
            "apply-custom-object" => {
                self.action_apply_custom_object();
            }
            "create-custom-object" => {
                self.action_create_custom_object();
            }
            "edit-custom-object" => {
                self.action_edit_custom_object();
            }
            "undo" => {
                // Our grabbed trigger is a bare 'z' (see styles.yaml), but
                // Inkscape's real undo shortcut is Ctrl+Z.
                if let Err(e) = self.press('z' as u32, CONTROL_MASK) {
                    println!("Failed to send undo to Inkscape window: {}", e);
                }
            }
            "delete" => {
                if let Err(e) = self.press(XK_DELETE, 0) {
                    println!("Failed to send delete to Inkscape window: {}", e);
                }
            }
            // "toggle-daemon" is handled directly in `listen`, since it
            // needs mutable access to the `active` flag that lives there
            // rather than on `self`.
            _ => {}
        }
    }
}
