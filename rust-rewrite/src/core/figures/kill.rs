use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn execute_kill(daemon: &str) {
    let pid_dir = PathBuf::from("/tmp/lesson-manager");
    let pid_file = pid_dir.join("watch.pid");

    if !pid_file.exists() {
        println!("No active watch daemon found (PID file missing).");
        return;
    }

    let pid_str = match fs::read_to_string(&pid_file) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            println!("Failed to read watch.pid");
            return;
        }
    };

    println!("Stopping watch daemon (PID: {})...", pid_str);

    // Send kill signal using standard OS kill command
    let status = Command::new("kill").arg("-9").arg(&pid_str).status();

    match status {
        Ok(s) if s.success() => {
            println!("Watch daemon successfully killed.");
            let _ = fs::remove_file(&pid_file);
        }
        _ => {
            println!(
                "Failed to kill process PID {}. It may have already stopped.",
                pid_str
            );
            let _ = fs::remove_file(&pid_file);
        }
    }
}
