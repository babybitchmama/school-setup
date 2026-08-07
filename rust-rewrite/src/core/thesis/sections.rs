use std::fs;
use std::path::PathBuf;

use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;

use crate::config::LessonManagerConfigFile;

use crate::open_in_neovim;

use std::collections::HashMap;

use crate::rofi::input::prompt_input;

pub fn new_section(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    // Call our new clean helper!
    let Some(raw_input) = prompt_input("Section Note Topic", &config.rofi_options) else {
        println!("No input provided. Aborting.");
        return;
    };

    // Convert "Tensor Calculus" to "tensor-calculus"
    let sanitized_name = raw_input.to_lowercase().replace(' ', "-");
    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let sections_dir = thesis_path.join("notes").join("sections");

    if !sections_dir.exists() {
        fs::create_dir_all(&sections_dir).expect("Failed to create sections directory");
    }

    let target_path = sections_dir.join(format!("{}.tex", sanitized_name));

    if !target_path.exists() {
        fs::File::create(&target_path).expect("Failed to create section note file");
    }

    open_in_neovim(&sections_dir, &[target_path], terminal, editor, &config.editor_mode);
}

pub fn list_section_notes(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let terminal = &config.terminal;
    let editor = &config.editor;

    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let sections_dir = thesis_path.join("notes").join("sections");
    let pattern = format!("{}/*.tex", sections_dir.display());

    let mut display_list = Vec::new();
    let mut path_map = HashMap::new();

    if let Ok(entries) = glob::glob(&pattern) {
        for path in entries.flatten() {
            // Use file_stem() to drop the .tex extension
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Format "scalar-curvature" to "Scalar Curvature"
                let title_cased = stem
                    .split('-')
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                display_list.push(title_cased.clone());
                path_map.insert(title_cased, path);
            }
        }
    }

    if display_list.is_empty() {
        message(
            "No section notes found.",
            "info",
            &config.rofi_options,
            None,
        );
        return;
    }

    display_list.sort();

    // Use your updated select_from_rofi signature with the custom prompt
    if let Some(selected) = select_from_rofi(
        display_list,
        &config.rofi_options,
        "Select a section note".to_string(),
    ) {
        if let Some(target_path) = path_map.get(&selected) {
            open_in_neovim(&sections_dir, &[target_path.clone()], terminal, editor, &config.editor_mode);
        }
    }
}
