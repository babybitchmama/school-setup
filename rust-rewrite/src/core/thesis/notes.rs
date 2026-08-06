use chrono::Local;
use std::fs;
use std::path::PathBuf;

use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;

use crate::config::LessonManagerConfigFile;

use crate::open_in_neovim;

use std::collections::HashMap;

pub fn new_brain_dump(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let notes_dir = thesis_path.join("notes").join("brain-dump");

    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).expect("Failed to create brain-dump directory");
    }

    let today = Local::now().format("%m-%d-%Y").to_string();
    let target_path = notes_dir.join(format!("{}.tex", today));

    if !target_path.exists() {
        fs::File::create(&target_path).expect("Failed to create brain dump file");
    }

    open_in_neovim(&notes_dir, &[target_path], terminal, editor, &config.editor_mode);
}

pub fn list_brain_dump_files(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let brain_dump_dir = thesis_path.join("notes").join("brain-dump");
    let pattern = format!("{}/*.tex", brain_dump_dir.display());

    let mut display_list = Vec::new();
    let mut path_map = HashMap::new();

    if let Ok(entries) = glob::glob(&pattern) {
        for path in entries.flatten() {
            if let Some(filename) = path.file_stem().and_then(|f| f.to_str()) {
                display_list.push(filename.to_string());
                path_map.insert(filename.to_string(), path);
            }
        }
    }

    if display_list.is_empty() {
        message("No work notes found.", "info", &config.rofi_options, None);
        return;
    }

    display_list.sort_by(|a, b| b.cmp(a));

    if let Some(selected) = select_from_rofi(display_list, &config.rofi_options, "Select a work note:".to_string()) {
        if let Some(target_path) = path_map.get(&selected) {
            open_in_neovim(&brain_dump_dir, &[target_path.clone()], terminal, editor, &config.editor_mode);
        }
    }
}
