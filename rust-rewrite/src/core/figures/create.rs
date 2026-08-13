use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    ensure_directory, resolve_assignment_path, resolve_course_path, resolve_thesis_path,
};
use crate::config::LessonManagerConfigFile;
use crate::commands::{CreateTarget, SharedCreateArgs};

pub fn execute_create(config: &LessonManagerConfigFile, target: &CreateTarget) {
    let figures_path = match target {
        CreateTarget::Notes { course_name, .. } => {
            let base = resolve_course_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
        CreateTarget::Thesis { note_type, .. } => {
            match resolve_thesis_path(config, note_type) {
                Ok(base) => ensure_directory(&base, &config.figures_dir),
                Err(e) => { println!("❌ Error: {}", e); return; }
            }
        }
        CreateTarget::Assignments { course_name, .. } => {
            let base = resolve_assignment_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
    };

    let shared = match target {
        CreateTarget::Notes { shared, .. } => shared,
        CreateTarget::Thesis { shared, .. } => shared,
        CreateTarget::Assignments { shared, .. } => shared,
    };

    let final_name = prepare_figure_file(&figures_path, shared);
    crate::core::figures::copy::copy_template_to_clipboard(&config.figures, &final_name);

    if shared.tablet {
        super::spawn_tablet(&figures_path, Some(&final_name));
    } else {
        super::spawn_inkscape(&figures_path, Some(&final_name));
    }
}

fn prepare_figure_file(figures_dir: &Path, shared: &SharedCreateArgs) -> String {
    let file_name = match &shared.name {
        Some(n) => {
            if n.ends_with(".svg") {
                n.clone()
            } else {
                format!("{}.svg", n)
            }
        }
        None => {
            let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            format!("fig-{}.svg", since_epoch.as_secs())
        }
    };

    let file_path = figures_dir.join(&file_name);
    let minimal_svg = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><svg xmlns="http://www.w3.org/2000/svg" version="1.1"></svg>"#;

    if !file_path.exists() {
        if shared.no_template {
            let _ = fs::write(&file_path, minimal_svg);
            println!("📄 Created blank figure: {}", file_name);
        } else {
            let default_template = "~/.config/lesson-manager/figures/template.svg";
            let template_path = PathBuf::from(
                shellexpand::tilde(shared.template.as_deref().unwrap_or(default_template))
                    .to_string(),
            );

            if template_path.exists() && fs::copy(&template_path, &file_path).is_ok() {
                println!("📄 Created new figure from template: {}", file_name);
            } else {
                let _ = fs::write(&file_path, minimal_svg);
                println!("📄 Created blank fallback figure: {}", file_name);
            }
        }
    } else {
        println!("📂 Opening existing figure: {}", file_name);
    }

    file_name
}
