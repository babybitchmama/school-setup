use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{
    ensure_directory, find_master_tex, resolve_assignment_path, resolve_course_path,
    resolve_thesis_path,
};
use crate::commands::CopyTarget;
use crate::config::LessonManagerConfigFile;

pub fn execute_preview(config: &LessonManagerConfigFile, target: &CopyTarget) {
    // 1. Resolve the figures directory
    let figures_path = match target {
        CopyTarget::Notes { course_name, .. } => {
            let base = resolve_course_path(config, course_name.as_deref());
            ensure_directory(&base, &config.figures_dir)
        }
        CopyTarget::Thesis { note_type, .. } => match resolve_thesis_path(config, note_type) {
            Ok(base) => ensure_directory(&base, &config.figures_dir),
            Err(e) => {
                println!("❌ Error: {}", e);
                return;
            }
        },
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

    // 2. Select figure (via --name flag or Rofi)
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
                "❌ Figure '{}' not found in {}",
                name,
                figures_path.display()
            );
            None
        }
    } else {
        let svg_files = get_svg_filenames(&figures_path);
        if svg_files.is_empty() {
            println!("❌ No figures found in {}", figures_path.display());
            return;
        }
        crate::rofi::select::select_from_rofi(svg_files, &config.rofi_options, "Preview Figure".to_string())
    };

    let figure_file = match selected_figure {
        Some(f) if !f.is_empty() => f,
        _ => {
            println!("❌ No figure selected for preview.");
            return;
        }
    };

    // 3. Generate LaTeX snippet
    let name_only = figure_file.strip_suffix(".svg").unwrap_or(&figure_file);
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
        .figures
        .figure_template
        .iter()
        .map(|line| {
            line.replace("{name}", name_only)
                .replace("{caption}", &caption)
                .replace("{label}", &label)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 4. Locate master.tex automatically
    let master_path = match find_master_tex(&figures_path) {
        Some(path) => path,
        None => {
            println!(
                "❌ Could not find 'master.tex' by searching upward from {}",
                figures_path.display()
            );
            return;
        }
    };

    let workspace_dir = master_path.parent().unwrap();

    // 5. Stage files in /tmp/lesson-manager
    let tmp_base = PathBuf::from("/tmp/lesson-manager");
    if let Err(e) = fs::create_dir_all(&tmp_base) {
        println!("❌ Failed to create temporary directory: {}", e);
        return;
    }

    let folder_name = workspace_dir.file_name().unwrap_or_default();
    let staging_dir = tmp_base.join(folder_name);

    if let Err(e) = copy_dir_all(workspace_dir, &staging_dir) {
        println!("❌ Failed to copy workspace to staging directory: {}", e);
        return;
    }

    // 6. Replace document body in staged master.tex
    let staging_master_path = staging_dir.join("master.tex");
    let original_master_content = match fs::read_to_string(&staging_master_path) {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Failed to read staged master.tex: {}", e);
            return;
        }
    };

    let modified_content = replace_document_body(&original_master_content, &figure_snippet);

    if let Err(e) = fs::write(&staging_master_path, modified_content) {
        println!("❌ Failed to write snippet to staged master.tex: {}", e);
        return;
    }

    println!(
        "📄 Staged and isolated figure preview at: {}",
        staging_master_path.display()
    );

    // 7. Compile inside /tmp and open PDF
    compile_and_open(&staging_master_path, &staging_dir, &config.pdf_viewer);
}

fn replace_document_body(master_content: &str, snippet: &str) -> String {
    let begin_tag = "\\begin{document}";
    let end_tag = "\\end{document}";

    if let Some(begin_idx) = master_content.find(begin_tag) {
        let body_start = begin_idx + begin_tag.len();
        if let Some(end_rel_idx) = master_content[body_start..].find(end_tag) {
            let body_end = body_start + end_rel_idx;

            let mut new_content = String::with_capacity(master_content.len());
            new_content.push_str(&master_content[..body_start]);
            new_content.push_str("\n\n");
            new_content.push_str(snippet);
            new_content.push_str("\n\n");
            new_content.push_str(&master_content[body_end..]);
            return new_content;
        }
    }

    format!("{}\n\n{}", master_content, snippet)
}

fn get_svg_filenames(figures_dir: &PathBuf) -> Vec<String> {
    fs::read_dir(figures_dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "svg"))
                .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn compile_and_open(master_file: &Path, work_dir: &Path, pdf_viewer: &String) {
    println!("⚙️ Compiling isolated preview document from /tmp...");

    let status = Command::new("pdflatex")
        .arg(master_file)
        .current_dir(work_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✅ Compilation successful!");
            let pdf_path = master_file.with_extension("pdf");
            println!("📱 Opening PDF: {}", pdf_path.display());
            let _ = Command::new(&pdf_viewer).arg(pdf_path).spawn();
        }
        _ => {
            println!("❌ Compilation failed. Check your tectonic/latex logs above.");
        }
    }
}
