use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{
    ensure_directory, resolve_assignment_path, resolve_course_path, resolve_thesis_path,
};
use crate::commands::CopyTarget;
use crate::config::LessonManagerConfigFile;
use crate::core::figures::get_svg_filenames;

pub fn execute_preview(config: &LessonManagerConfigFile, target: &CopyTarget) {
    // 1. Resolve the target figures directory
    let figures_path = match target {
        CopyTarget::Notes { course_name, .. } => {
            let base = resolve_course_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
        CopyTarget::Thesis { note_type, .. } => match resolve_thesis_path(config, note_type) {
            Ok(base) => ensure_directory(&base, &config.figures_dir),
            Err(e) => { println!("Error: {}", e); return; }
        },
        CopyTarget::Assignments { course_name, .. } => {
            let base = resolve_assignment_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
    };

    println!("{}", figures_path.display());

    let shared = match target {
        CopyTarget::Notes { shared, .. } => shared,
        CopyTarget::Thesis { shared, .. } => shared,
        CopyTarget::Assignments { shared, .. } => shared,
    };

    // 2. Select the figure (via --name flag or Rofi selection)
    let selected_figure = if let Some(explicit_name) = &shared.name {
        let name = if explicit_name.ends_with(".svg") {
            explicit_name.clone()
        } else {
            format!("{}.svg", explicit_name)
        };
        if figures_path.join(&name).exists() {
            Some(name)
        } else {
            println!(
                "Figure '{}' not found in {}",
                name,
                figures_path.display()
            );
            None
        }
    } else {
        let svg_files = get_svg_filenames(&figures_path);
        if svg_files.is_empty() {
            println!("No figures found in {}", figures_path.display());
            return;
        }
        crate::rofi::select::select_from_rofi(svg_files, &config.rofi_options, "Preview Figure".to_string())
    };

    let figure_file = match selected_figure {
        Some(f) if !f.is_empty() => f,
        _ => {
            println!("No figure selected for preview.");
            return;
        }
    };

    let name_only = figure_file.strip_suffix(".svg").unwrap_or(&figure_file);

    // 3. Generate the LaTeX snippet using config template
    let caption = name_only
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let label = format!("fig:{}", name_only);

    let figure_snippet = config
        .figure_template
        .iter()
        .map(|line| {
            line.replace("{name}", name_only)
                .replace("{caption}", &caption)
                .replace("{label}", &label)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 4. Set up the isolated staging directory in /tmp
    // Format: /tmp/lesson-manager/preview/<target-type>-<figure-name>/
    let target_type_str = match target {
        CopyTarget::Notes { .. } => "notes",
        CopyTarget::Thesis { .. } => "thesis",
        CopyTarget::Assignments { .. } => "assignment",
    };

    let staging_dir = PathBuf::from(format!(
        "/tmp/lesson-manager/preview/{}-{}",
        target_type_str, name_only
    ));
    if staging_dir.exists() {
        let _ = fs::remove_dir_all(&staging_dir);
    }

    let staging_figures_dir = staging_dir.join("figures");
    if let Err(e) = fs::create_dir_all(&staging_figures_dir) {
        println!("Failed to create staging directory: {}", e);
        return;
    }

// 5. Copy ONLY files matching the selected figure basename (e.g., figure-1.*)
    if let Err(e) = copy_matching_figure_files(&figures_path, &staging_figures_dir, name_only) {
        println!("Failed to copy figure assets to staging directory: {}", e);
        return;
    }

    // 6. Locate and load the global preview template
    let template_path =
        shellexpand::tilde("~/.config/lesson-manager/figures/preview-template.tex").into_owned();
    let template_content = match fs::read_to_string(&template_path) {
        Ok(c) => c,
        Err(_) => {
            // Fallback default template if file doesn't exist yet
            format!(
                r#"\documentclass{{article}}
\usepackage{{import}}
\usepackage{{graphicx}}
\usepackage{{xcolor}}
\usepackage{{amsmath,amssymb,amsfonts}}

\newcommand{{\incfig}}[2][1]{{%
    \def\svgwidth{{#1\columnwidth}}%
    \import{{./figures/}}{{#2.pdf_tex}}%
}}

\begin{{document}}
\pagestyle{{empty}}
\centering

{{figure_snippet}}

\end{{document}}
"#
            )
        }
    };

    // Replace placeholder with actual figure snippet
    let master_content = template_content.replace("{figure_snippet}", &figure_snippet);
    let staging_master_path = staging_dir.join("master.tex");

    if let Err(e) = fs::write(&staging_master_path, master_content) {
        println!("Failed to write staged master.tex: {}", e);
        return;
    }

    println!("Staged preview at: {}", staging_master_path.display());

    // 7. Compile locally with pdflatex and open resulting PDF
    compile_and_open(&staging_master_path, &staging_dir, &config.pdf_viewer);
}

/// Copies only files whose stems match the target figure name (e.g., figure-1.svg, figure-1.pdf_tex, etc.)
fn copy_matching_figure_files(src_dir: &Path, dst_dir: &Path, base_name: &str) -> std::io::Result<()> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == base_name {
                    if let Some(file_name) = path.file_name() {
                        fs::copy(&path, dst_dir.join(file_name))?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn compile_and_open(master_file: &Path, work_dir: &Path, pdf_viewer: &String) {
    println!("Compiling preview document with pdflatex...");

    // Run pdflatex twice to ensure correct rendering/references
    for pass in 1..=2 {
        let status = Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg(master_file)
            .current_dir(work_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                if pass == 2 {
                    println!("Compilation successful!");
                    let pdf_path = master_file.with_extension("pdf");
                    println!("Opening PDF: {}", pdf_path.display());
                    let _ = Command::new(pdf_viewer).arg(pdf_path).spawn();
                }
            }
            _ => {
                println!(
                    "Compilation failed on pass {}. Check your pdflatex log output above.",
                    pass
                );
                return;
            }
        }
    }
}
