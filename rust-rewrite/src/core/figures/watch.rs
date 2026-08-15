use std::{fs, path::Path};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;

use crate::config::LessonManagerConfigFile;
use notify::{Event, EventKind, RecursiveMode, Result, Watcher};

pub fn execute_watch(config: &LessonManagerConfigFile) {
    let watch_path = shellexpand::tilde(&config.notes_dir).into_owned();
    let path = PathBuf::from(&watch_path);

    if !path.exists() {
        println!("Watch directory does not exist: {}", path.display());
        return;
    }

    daemonize_process(&path);

    if let Err(e) = run_watcher(&path) {
        println!("Watcher error: {:?}", e);
    }
}

fn daemonize_process(watch_path: &Path) {
    let pid = std::process::id();
    let pid_dir = PathBuf::from("/tmp/lesson-manager");
    let _ = fs::create_dir_all(&pid_dir);
    let pid_file = pid_dir.join("watch.pid");

    if let Err(e) = fs::write(&pid_file, pid.to_string()) {
        println!("Failed to write watch PID file: {}", e);
    } else {
        println!(
            "Starting figure compilation daemon (PID: {}) watching: {}",
            pid,
            watch_path.display()
        );
    }
}

fn run_watcher(watch_path: &PathBuf) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event>| match res {
        Ok(event) => {
            let _ = tx.send(event);
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(watch_path, RecursiveMode::Recursive)?;

    println!("Watching for .svg file modifications...");

    loop {
        match rx.recv() {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths {
                        if path.extension().is_some_and(|ext| ext == "svg") {
                            compile_svg(&path);
                        }
                    }
                }
            }
            Err(e) => {
                println!("watch recv error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

fn compile_svg(svg_path: &PathBuf) {
    let stem = match svg_path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return,
    };

    let parent_dir = match svg_path.parent() {
        Some(p) => p,
        None => return,
    };

    let pdf_path = parent_dir.join(format!("{}.pdf", stem));

    let status = Command::new("inkscape")
        .arg(format!("--export-filename={}", pdf_path.display()))
        .arg("--export-latex")
        .arg(svg_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Successfully recompiled: {}.svg -> .pdf & .pdf_tex", stem);
        }
        Ok(_) => {
            println!("Inkscape export failed for: {}", svg_path.display());
        }
        Err(e) => {
            println!(
                "Failed to execute Inkscape binary: {}. Is Inkscape installed?",
                e
            );
        }
    }
}
