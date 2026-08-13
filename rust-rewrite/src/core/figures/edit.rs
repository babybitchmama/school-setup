use super::{ensure_directory, resolve_assignment_path, resolve_course_path, resolve_thesis_path};
use crate::commands::EditTarget;
use crate::config::LessonManagerConfigFile;
use crate::core::figures::get_svg_filenames;

pub fn execute_edit(config: &LessonManagerConfigFile, target: &EditTarget) {
    // 1. Resolve the figures directory using our standard path matchers
    let figures_path = match target {
        EditTarget::Notes { course_name, .. } => {
            let base = resolve_course_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
        EditTarget::Thesis { note_type, .. } => match resolve_thesis_path(config, note_type) {
            Ok(base) => ensure_directory(&base, &config.figures_dir),
            Err(e) => {
                println!("❌ Error: {}", e);
                return;
            }
        },
        EditTarget::Assignments { course_name, .. } => {
            let base = resolve_assignment_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
    };

    // 2. Extract shared arguments (like tablet flag)
    let shared = match target {
        EditTarget::Notes { shared, .. } => shared,
        EditTarget::Thesis { shared, .. } => shared,
        EditTarget::Assignments { shared, .. } => shared,
    };

    // 3. Scan directory for available .svg files
    let svg_files = get_svg_filenames(&figures_path);
    if svg_files.is_empty() {
        println!("❌ No figures found in {}", figures_path.display());
        return;
    }

    // 4. Prompt Rofi using your project's native select function
    match crate::rofi::select::select_from_rofi(
        svg_files,
        &config.rofi_options,
        "Edit Figure".to_string(),
    ) {
        Some(selected) if !selected.is_empty() => {
            if shared.tablet {
                super::spawn_tablet(&figures_path, Some(&selected));
            } else {
                super::spawn_inkscape(&figures_path, Some(&selected));
            }
        }
        _ => {
            println!("❌ No figure selected.");
        }
    }
}
