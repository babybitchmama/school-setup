use std::io::Write;
use std::process::{Command, Stdio};

pub fn select_from_rofi(options: Vec<String>, rofi_options: &[String]) -> Option<String> {
    let rofi_input = options.join("\n");

    let mut child = Command::new("rofi")
        .arg("-dmenu")
        .args(rofi_options)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start rofi");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(rofi_input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    if output.status.success() {
        let selected_option = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(selected_option)
    } else {
        None
    }
}
