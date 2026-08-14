use std::io::Write;
use std::process::{Command, Stdio};

use super::{
    ensure_directory, resolve_assignment_path, resolve_course_path, resolve_thesis_path,
};
use crate::commands::CopyTarget;
use crate::config::LessonManagerConfigFile;
use crate::core::figures::get_svg_filenames;

pub fn execute_copy(config: &LessonManagerConfigFile, target: &CopyTarget) {
    let figures_path = match target {
        CopyTarget::Notes { course_name, .. } => {
            let base = resolve_course_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
        CopyTarget::Thesis { note_type, .. } => {
            match resolve_thesis_path(config, note_type) {
                Ok(base) => ensure_directory(&base, &config.figures_dir),
                Err(e) => { println!("Error: {}", e); return; }
            }
        }
        CopyTarget::Assignments { course_name, .. } => {
            let base = resolve_assignment_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
    };

    let shared = match target {
        CopyTarget::Notes { shared, .. } => shared,
        CopyTarget::Thesis { shared, .. } => shared,
        CopyTarget::Assignments { shared, .. } => shared,
    };

    if let Some(explicit_name) = &shared.name {
        let file_name = if explicit_name.ends_with(".svg") {
            explicit_name.clone()
        } else {
            format!("{}.svg", explicit_name)
        };
        if figures_path.join(&file_name).exists() {
            copy_template_to_clipboard(&config.figure_template, &file_name);
        } else {
            println!("Figure '{}' not found.", file_name);
        }
        return;
    }

    let svg_files = get_svg_filenames(&figures_path);
    if svg_files.is_empty() {
        println!("No figures found in {}", figures_path.display());
        return;
    }

    if let Some(selected) =
        crate::rofi::select::select_from_rofi(svg_files, &config.rofi_options, "Select Figure".to_string())
    {
        if !selected.is_empty() {
            copy_template_to_clipboard(&config.figure_template, &selected);
        }
    }
}

pub fn copy_template_to_clipboard(figure_template: &Vec<String>, file_name: &str) {
    let name_only = file_name.strip_suffix(".svg").unwrap_or(file_name);

    let caption = name_only
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let label = format!("fig:{}", name_only);

    let final_string = &figure_template
        .iter()
        .map(|line| {
            line.replace("{name}", name_only)
                .replace("{caption}", &caption)
                .replace("{label}", &label)
        })
        .collect::<Vec<_>>()
        .join("\n");

    if let Ok(mut child) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(final_string.as_bytes());
        }
        let _ = child.wait();
        println!("📋 Copied to clipboard:\n{}", final_string);
    } else {
        println!("Failed to execute xclip.");
    }
}
