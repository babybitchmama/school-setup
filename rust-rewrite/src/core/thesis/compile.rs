use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::LessonManagerConfigFile;

pub fn compile_targets(config: &LessonManagerConfigFile, targets: &[String]) {
    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());

    for target in targets {
        println!("Preparing target: {}", target);

        let note_config = match config.thesis_note_types.get(target) {
            Some(cfg) => cfg,
            None => {
                println!(
                    "Warning: Target '{}' not found in config. Skipping.",
                    target
                );
                continue;
            }
        };

        let target_dir = thesis_path.join(&note_config.path);
        let master_path = target_dir.join("master.tex");

        if !master_path.exists() {
            println!(
                "Error: No master.tex found in {}. Skipping.",
                target_dir.display()
            );
            continue;
        }

        let content_dir = match &note_config.folder {
            Some(folder_name) => target_dir.join(folder_name),
            None => target_dir.clone(),
        };

        if !content_dir.exists() {
            fs::create_dir_all(&content_dir).expect("Failed to create content directory");
        }

        let build_dir = PathBuf::from(format!("/tmp/lesson-manager-build/{}", target));
        if !build_dir.exists() {
            fs::create_dir_all(&build_dir).expect("Failed to create build directory");
        }
        let build_file = build_dir.join("tmp.tex");

        let inputs = gather_inputs(&content_dir, &note_config.style);

        if inputs.is_empty() {
            println!("No valid .tex files found for {}. Skipping.", target);
            continue;
        }

        match inject_inputs(&master_path, &build_file, &inputs) {
            Ok(_) => {
                execute_ghost_build(&target_dir, &build_dir, &build_file, &config.pdf_viewer);
            }
            Err(e) => {
                println!("Error slicing master.tex for {}: {}", target, e);
            }
        }
    }
}

fn gather_inputs(content_dir: &Path, style: &str) -> Vec<String> {
    let mut files_to_include = Vec::new();

    if style == "single" {
        if let Ok(entries) = fs::read_dir(content_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("tex") {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                    if !file_name.starts_with('_') && file_name != "master.tex" && file_name != "preamble.tex" {
                        files_to_include.push(path);
                    }
                }
            }
        }
        files_to_include.sort();
    } else if style == "folder" {
        let mut folders = Vec::new();

        if let Ok(entries) = fs::read_dir(content_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().unwrap_or_default().to_string_lossy();

                    if !folder_name.starts_with('_') && folder_name != "templates" {
                        folders.push(path);
                    }
                }
            }
        }
        folders.sort();

        for folder in folders {
            let mut inner_files = Vec::new();
            if let Ok(entries) = fs::read_dir(&folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("tex") {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                        if !file_name.starts_with('_') && file_name != "master.tex" {
                            inner_files.push(path);
                        }
                    }
                }
            }
            inner_files.sort();
            files_to_include.extend(inner_files);
        }
    }

    files_to_include
        .into_iter()
        .map(|path| format!("  \\input{{{}}}", path.display()))
        .collect()
}

fn inject_inputs(master_path: &Path, build_file: &Path, inputs: &[String]) -> Result<(), String> {
    let master_content =
        fs::read_to_string(master_path).map_err(|e| format!("Failed to read master.tex: {}", e))?;

    let begin_marker = "% BEGIN SECTION INPUT";
    let end_marker = "% END SECTION INPUT";

    let begin_idx = master_content
        .find(begin_marker)
        .ok_or_else(|| "Could not find '% BEGIN SECTION INPUT' marker in master.tex".to_string())?;

    let end_idx = master_content
        .find(end_marker)
        .ok_or_else(|| "Could not find '% END SECTION INPUT' marker in master.tex".to_string())?;

    if begin_idx >= end_idx {
        return Err("Markers are out of order!".to_string());
    }

    let preamble = &master_content[..begin_idx + begin_marker.len()];
    let postamble = &master_content[end_idx..];

    let final_tex = format!("{}\n{}\n  {}", preamble, inputs.join("\n"), postamble);

    fs::write(build_file, final_tex)
        .map_err(|e| format!("Failed to write tmp.tex to /tmp: {}", e))?;

    Ok(())
}

fn execute_ghost_build(target_dir: &Path, build_dir: &Path, build_file: &Path, pdf_viewer: &str) {
    println!("Compiling ghost build in {}...", build_dir.display());

    let status = Command::new("latexmk")
        .current_dir(target_dir)
        .arg("-pdf")
        .arg("-interaction=nonstopmode")
        .arg(format!("-output-directory={}", build_dir.display()))
        .arg(build_file)
        .status()
        .expect("Failed to execute latexmk. Is it installed?");

    if status.success() {
        let pdf_path = build_dir.join("tmp.pdf");
        println!("Compilation successful! Opening in {}...", pdf_viewer);

        Command::new(pdf_viewer)
            .arg(&pdf_path)
            .spawn()
            .expect("Failed to open PDF viewer");
    } else {
        println!(
            "Compilation failed. Check the logs in {}",
            build_dir.display()
        );
    }
}
