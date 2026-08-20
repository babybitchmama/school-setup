use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{LessonManagerConfigFile, NoteTypeConfig};
use crate::open_in_neovim;
use crate::rofi::input::prompt_input;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;
use chrono::Local;

pub fn create_note(
    config: &LessonManagerConfigFile,
    note_type: &str,
    provided_name: Option<String>,
) {
    let note_config = match config.thesis_note_types.get(note_type) {
        Some(cfg) => cfg,
        None => {
            message(
                &format!("Error: '{}' not defined", note_type),
                "error",
                &config.rofi_options,
                None,
            );
            return;
        }
    };

    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());
    let target_dir = thesis_path.join(&note_config.path);

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).expect("Failed to create directory");
    }

    let initial_input = provided_name.unwrap_or_else(|| {
        note_config
            .naming
            .clone()
            .unwrap_or_else(|| "prompt".to_string())
    });

    let (actual_style, string_to_evaluate) = if let Some(rest) = initial_input.strip_prefix("file:")
    {
        ("single".to_string(), rest.to_string())
    } else if let Some(rest) = initial_input.strip_prefix("folder:") {
        ("folder".to_string(), rest.to_string())
    } else {
        (note_config.style.clone(), initial_input)
    };

    let raw_name = if string_to_evaluate == "prompt" {
        let prompt_text = format!("{} Name", note_type.replace('-', " ").to_uppercase());
        let Some(input) = prompt_input(&prompt_text, &config.rofi_options) else {
            return;
        };
        input
    } else if let Some(format_str) = string_to_evaluate.strip_prefix("date:") {
        Local::now().format(format_str).to_string()
    } else {
        string_to_evaluate
    };

    let clean_name = raw_name.trim().to_lowercase().replace(' ', "-");
    if clean_name.is_empty() {
        return;
    }

    let base_name = Local::now().format(&clean_name).to_string();

    if actual_style == "single" {
        let file_path = target_dir.join(format!("{}.tex", base_name));
        if !file_path.exists() {
            fs::File::create(&file_path).expect("Failed to create file");
        }
        open_in_neovim(
            &target_dir,
            &[file_path],
            &config.terminal,
            &config.editor,
            &String::from("n"),
            None,
        );
    } else if actual_style == "folder" {
        let folder_dir = target_dir.join(&base_name);
        if !folder_dir.exists() {
            fs::create_dir_all(&folder_dir).expect("Failed to create folder");
        }

        let default_files = vec!["notes".to_string()];
        let files_to_create = note_config.files.as_ref().unwrap_or(&default_files);
        let mut files_to_open = Vec::new();

        for file_name in files_to_create {
            let time_formatted = Local::now().format(file_name).to_string();
            let final_file_name = time_formatted.replace("{base_name}", &base_name);
            let file_path = folder_dir.join(format!("{}.tex", final_file_name));
            if !file_path.exists() {
                fs::File::create(&file_path).expect("Failed to create file");
            }
            files_to_open.push(file_path);
        }

        open_in_neovim(
            &folder_dir,
            &files_to_open,
            &config.terminal,
            &config.editor,
            &String::from("n"),
            None,
        );
    }
}

pub fn list_notes(config: &LessonManagerConfigFile, note_type: &str) {
    let note_config = match config.thesis_note_types.get(note_type) {
        Some(cfg) => cfg,
        None => {
            message(
                &format!("Error: '{}' not defined", note_type),
                "error",
                &config.rofi_options,
                None,
            );
            return;
        }
    };

    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());
    let target_dir = thesis_path.join(&note_config.path);

    let content_dir = match &note_config.folder {
        Some(folder_name) => target_dir.join(folder_name),
        None => target_dir.clone(),
    };

    if !content_dir.exists() {
        fs::create_dir_all(&content_dir).expect("Failed to create directory");
    }

    let prompt_title = note_type.replace('-', " ");

    // Pass content_dir as the starting point for interactive navigation
    interactive_navigate(
        config,
        note_config,
        &content_dir,
        &thesis_path,
        &prompt_title,
    );
}

fn interactive_navigate(
    config: &LessonManagerConfigFile,
    note_config: &NoteTypeConfig,
    current_dir: &Path,
    thesis_path: &Path,
    prompt_title: &str,
) {
    let mut files = Vec::new();
    let mut folders = Vec::new();
    let mut path_map = HashMap::new();
    let mut is_dir_map = HashMap::new();

    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    if folder_name != "templates" {
                        let display_name = format!("{}/", folder_name);
                        folders.push(display_name.clone());
                        path_map.insert(display_name.clone(), path.clone());
                        is_dir_map.insert(display_name, true);
                    }
                }
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("tex") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "master" {
                        files.push(stem.to_string());
                        path_map.insert(stem.to_string(), path.clone());
                        is_dir_map.insert(stem.to_string(), false);
                    }
                }
            }
        }
    }

    files.sort_by(|a, b| b.cmp(a));
    folders.sort_by(|a, b| b.cmp(a));

    let mut display_list = files;
    display_list.extend(folders);

    let relative_path = current_dir.strip_prefix(thesis_path).unwrap_or(current_dir);

    let prompt_text = if current_dir == thesis_path.join(&note_config.path) {
        format!(
            "Select or Create {} ({})",
            prompt_title,
            relative_path.display()
        )
    } else {
        format!("Inside {}/", relative_path.display())
    };

    if let Some(selected) = select_from_rofi(display_list, &config.rofi_options, prompt_text) {
        if let Some(target_path) = path_map.get(&selected) {
            let is_folder = is_dir_map.get(&selected).unwrap_or(&false);

            if *is_folder {
                interactive_navigate(config, note_config, target_path, thesis_path, prompt_title);
            } else {
                open_in_neovim(
                    current_dir,
                    &[target_path.clone()],
                    &config.terminal,
                    &config.editor,
                    &String::from("n"),
                    None,
                );
            }
        } else {
            handle_interactive_creation(
                config,
                note_config,
                current_dir,
                thesis_path,
                &selected,
                prompt_title,
            );
        }
    }
}

fn handle_interactive_creation(
    config: &LessonManagerConfigFile,
    note_config: &NoteTypeConfig,
    current_dir: &Path,
    thesis_path: &Path,
    input: &str,
    prompt_title: &str,
) {
    let (actual_style, string_to_evaluate) = if let Some(rest) = input.strip_prefix("file:") {
        ("single".to_string(), rest.to_string())
    } else if let Some(rest) = input.strip_prefix("folder") {
        ("folder".to_string(), rest.to_string())
    } else {
        (note_config.style.clone(), input.to_string())
    };

    let raw_name = if string_to_evaluate == "prompt" {
        let prompt_text = format!("{} Name", prompt_title);
        let Some(ans) = prompt_input(&prompt_text, &config.rofi_options) else {
            return;
        };
        ans
    } else if let Some(format_str) = string_to_evaluate.strip_prefix("date:") {
        Local::now().format(format_str).to_string()
    } else {
        string_to_evaluate
    };

    let clean_name = raw_name.trim().to_lowercase().replace(' ', "-");
    if clean_name.is_empty() {
        return;
    }

    let base_name = Local::now().format(&clean_name).to_string();

    if actual_style == "single" {
        let file_path = current_dir.join(format!("{}.tex", base_name));
        if !file_path.exists() {
            fs::File::create(&file_path).expect("Failed to create file");
        }
        open_in_neovim(
            current_dir,
            &[file_path],
            &config.terminal,
            &config.editor,
            &String::from("n"),
            None,
        );
    } else if actual_style == "folder" {
        let folder_dir = current_dir.join(&base_name);
        if !folder_dir.exists() {
            fs::create_dir_all(&folder_dir).expect("Failed to create folder");
        }

        let default_files = vec!["notes".to_string()];
        let files_to_create = note_config.files.as_ref().unwrap_or(&default_files);

        for file_name in files_to_create {
            let time_formatted = Local::now().format(file_name).to_string();
            let final_file_name = time_formatted.replace("{base_name}", &base_name);
            let file_path = folder_dir.join(format!("{}.tex", final_file_name));

            if !file_path.exists() {
                fs::File::create(&file_path).expect("Failed to create file");
            }
        }

        interactive_navigate(config, note_config, &folder_dir, thesis_path, prompt_title);
    } else {
        message(
            &format!("Unknown style '{}'", actual_style),
            "error",
            &config.rofi_options,
            None,
        );
    }
}
