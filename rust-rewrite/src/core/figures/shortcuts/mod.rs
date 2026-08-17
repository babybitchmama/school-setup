pub mod clipboard;
pub mod config;
pub mod constants;
pub mod manager;
pub mod math;
pub mod normal;

use std::fs;
use std::path::PathBuf;
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, Window,
};

use crate::config::LessonManagerConfigFile;
use config::StylesConfig;
use constants::PID_FILE;
use manager::Manager;

pub fn execute_shortcuts() {
    let styles = StylesConfig::load();
    let app_config = LessonManagerConfigFile::load();

    daemonize();

    if let Err(e) = watch_for_inkscape_windows(styles, app_config) {
        println!("Shortcuts daemon error: {}", e);
    }
}

fn daemonize() {
    let pid = std::process::id();
    let pid_path = PathBuf::from(PID_FILE);
    if let Some(dir) = pid_path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Err(e) = fs::write(&pid_path, pid.to_string()) {
        println!("Failed to write shortcuts PID file: {}", e);
    } else {
        println!("Starting Inkscape shortcuts daemon (PID: {})...", pid);
    }
}

fn is_inkscape(conn: &impl Connection, window: Window) -> bool {
    let Ok(cookie) = conn.get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::ANY, 0, 1024)
    else {
        return false;
    };
    let Ok(reply) = cookie.reply() else {
        return false;
    };
    String::from_utf8_lossy(&reply.value)
        .split('\u{0}')
        .any(|part| part.to_lowercase().contains("inkscape"))
}

fn spawn_manager(window: Window, styles: StylesConfig, app_config: LessonManagerConfigFile) {
    thread::spawn(move || match Manager::new(window, app_config) {
        Ok(manager) => {
            println!("Watching Inkscape window {}", window);
            if let Err(e) = manager.listen(&styles) {
                println!("Manager for window {} exited: {}", window, e);
            }
        }
        Err(e) => println!("Failed to attach to window {}: {}", window, e),
    });
}

fn watch_for_inkscape_windows(
    styles: StylesConfig,
    app_config: LessonManagerConfigFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    let tree = conn.query_tree(root)?.reply()?;
    for child in tree.children {
        if is_inkscape(&conn, child) {
            println!("Found existing Inkscape window");
            spawn_manager(child, styles.clone(), app_config.clone());
        }
    }

    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::SUBSTRUCTURE_NOTIFY),
    )?
    .check()?;

    loop {
        let event = conn.wait_for_event()?;
        if let Event::CreateNotify(ev) = event {
            if is_inkscape(&conn, ev.window) {
                println!("New Inkscape window detected");
                spawn_manager(ev.window, styles.clone(), app_config.clone());
            }
        }
    }
}
