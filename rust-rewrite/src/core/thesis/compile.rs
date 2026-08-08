use crate::config::LessonManagerConfigFile;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn generate_and_compile(config: &LessonManagerConfigFile, targets: &[String]) {
    let thesis_path = PathBuf::from(shellexpand::tilde(&config.thesis_dir).to_string());

    // Create the /tmp build directory
    let build_dir = PathBuf::from("/tmp/lesson-manager-build");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir).expect("Failed to create /tmp build directory");
    }

    // We will compile each target as its own standalone PDF using its local master.tex
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

        // 1. Gather all the \input{} paths for this specific target
        let pattern = format!("{}/**/*.tex", target_dir.display());
        let mut files_for_target = Vec::new();

        if let Ok(entries) = glob::glob(&pattern) {
            for path in entries.flatten() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Make sure we don't accidentally \input the master.tex into itself!
                if file_name != "master.tex" {
                    files_for_target.push(format!("\\input{{{}}}", path.display()));
                }
            }
        }

        if files_for_target.is_empty() {
            println!("No .tex files found in {}. Skipping.", target_dir.display());
            continue;
        }

        files_for_target.sort();

        // 2. Read the local master.tex
        let local_master_path = target_dir.join("master.tex");
        let master_content = if local_master_path.exists() {
            fs::read_to_string(&local_master_path).expect("Failed to read local master.tex")
        } else {
            println!(
                "Warning: No master.tex found in {}. Generating default.",
                target_dir.display()
            );
            "\\documentclass{article}\n\\begin{document}\n\\end{document}".to_string()
        };

        // 3. Slice the master_content to inject the inputs safely in memory
        let begin_doc = "\\begin{document}";
        let end_doc = "\\end{document}";

        let final_tex = if let (Some(begin_idx), Some(end_idx)) =
            (master_content.find(begin_doc), master_content.find(end_doc))
        {
            // Grab everything up to and including \begin{document}
            let preamble = &master_content[..begin_idx + begin_doc.len()];
            // Grab \end{document} and everything after
            let postamble = &master_content[end_idx..];

            format!(
                "{}\n\n{}\n\n{}",
                preamble,
                files_for_target.join("\n"),
                postamble
            )
        } else {
            println!(
                "Error: Could not find \\begin{{document}} and \\end{{document}} in {}. Skipping.",
                local_master_path.display()
            );
            continue;
        };

        // 4. Write to /tmp and Compile
        let build_file = build_dir.join(format!("{}_master.tex", target));
        fs::write(&build_file, final_tex).expect("Failed to write to temp directory");

        println!("Compiling {} in /tmp/lesson-manager-build...", target);

        let status = Command::new("latexmk")
            .current_dir(&target_dir) // Run from the target dir so local paths/images work!
            .arg("-pdf")
            .arg("-interaction=nonstopmode")
            .arg(format!("-output-directory={}", build_dir.display()))
            .arg(&build_file)
            .status()
            .expect("Failed to execute latexmk");

        if status.success() {
            let pdf_path = build_dir.join(format!("{}_master.pdf", target));
            println!(
                "Successfully compiled {}! Opening in {}...",
                target, config.pdf_viewer
            );

            Command::new(&config.pdf_viewer)
                .arg(&pdf_path)
                .spawn()
                .expect("Failed to open PDF viewer");
        } else {
            println!(
                "Compilation failed for {}. Check logs in /tmp/lesson-manager-build/",
                target
            );
        }
    }
}
