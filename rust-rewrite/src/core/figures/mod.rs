pub mod copy;
pub mod create;
pub mod edit;
pub mod kill;
pub mod preview;
pub mod shortcuts;
pub mod watch;

use crate::FigureCommands;
use crate::config::LessonManagerConfigFile;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn main(config: &LessonManagerConfigFile, command: &FigureCommands) {
    match command {
        FigureCommands::Create { target } => create::execute_create(config, target),
        FigureCommands::Copy { target } => copy::execute_copy(config, target),
        FigureCommands::Edit { target } => edit::execute_edit(config, target),
        FigureCommands::Preview { target } => preview::execute_preview(config, target),
        FigureCommands::Watch {} => watch::execute_watch(config),
        FigureCommands::Shortcuts => shortcuts::execute_shortcuts(),
        FigureCommands::Kill { daemon } => {
            let target = daemon.clone().unwrap_or_else(|| "both".to_string());
            kill::execute_kill(&target);
        }
    }
}

// ==========================================
// PATH RESOLVERS & UTILITIES
// ==========================================

/// Resolves the base course directory (e.g. ~/Documents/school-notes/current-course or specific course)
pub fn resolve_course_path(config: &LessonManagerConfigFile, course_name: Option<&str>) -> PathBuf {
    let root_expanded = shellexpand::tilde(&config.notes_dir).to_string();

    match course_name {
        Some(name) => PathBuf::from(root_expanded).join(name),
        None => PathBuf::from(shellexpand::tilde(&config.current_course).to_string()),
    }
}

/// Resolves the thesis directory based on note_type
pub fn resolve_thesis_path(
    config: &LessonManagerConfigFile,
    note_type: &str,
) -> Result<PathBuf, String> {
    let note_config = config
        .thesis_note_types
        .get(note_type)
        .ok_or_else(|| format!("Note type '{}' not defined in config.", note_type))?;

    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());
    let target_dir = thesis_path.join(&note_config.path);

    let content_dir = match &note_config.folder {
        Some(folder_name) => target_dir.join(folder_name),
        None => target_dir,
    };

    if !content_dir.exists() {
        fs::create_dir_all(&content_dir)
            .map_err(|e| format!("Failed to create thesis directory: {}", e))?;
    }

    Ok(content_dir)
}

/// Resolves the figure directory for assignments using the configured assignment folder name
pub fn resolve_assignment_path(
    config: &LessonManagerConfigFile,
    course_name: Option<&str>,
) -> PathBuf {
    let course_path = resolve_course_path(config, course_name);
    course_path
        .join(&config.assignments_dir)
        .join(&config.assignments_root)
}

/// Ensures the figures directory exists on disk and returns its path
pub fn ensure_directory(base_dir: &Path, figures_dir: &String) -> PathBuf {
    let figures_dir = base_dir.join(&figures_dir);
    if !figures_dir.exists() {
        fs::create_dir_all(&figures_dir).expect("Failed to create figures directory");
    }
    figures_dir
}

/// Reads the figures directory and returns a sorted list of `.svg` filenames
fn get_svg_filenames(figures_dir: &PathBuf) -> Vec<String> {
    let entries = match fs::read_dir(figures_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "svg") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(name.to_string());
            }
        }
    }
    files.sort();
    files
}

// ==========================================
// PROCESS SPAWNERS
// ==========================================

pub fn spawn_inkscape(figures_dir: &Path, name: Option<&str>) {
    let mut cmd = Command::new("inkscape");
    let abs_dir = figures_dir
        .canonicalize()
        .unwrap_or_else(|_| figures_dir.to_path_buf());
    cmd.current_dir(&abs_dir);
    cmd.env("PWD", &abs_dir);

    if let Some(file_name) = name {
        let svg_name = if file_name.ends_with(".svg") {
            file_name.to_string()
        } else {
            format!("{}.svg", file_name)
        };
        let file_path = abs_dir.join(&svg_name);
        cmd.arg(&file_path);
        println!("Opening {} in Inkscape...", file_path.display());
    } else {
        println!(
            "Opening empty Inkscape instance in {}...",
            abs_dir.display()
        );
    }

    if let Err(e) = cmd.spawn() {
        println!("Failed to spawn Inkscape. Error: {}", e);
    }
}

pub fn spawn_tablet(figures_dir: &Path, name: Option<&str>) {
    println!("Tablet mode activated!");
    println!("Saving to: {}", figures_dir.display());
    if let Some(file_name) = name {
        println!("Target file: {}", file_name);
    }
}
