use std::process::Command;

pub fn prompt_input(prompt: &str, rofi_options: &[String]) -> Option<String> {
    let output = Command::new("rofi")
        .arg("-dmenu")
        .arg("-p")
        .arg(prompt)
        .arg("-lines")
        .arg("0") // Hides the dropdown list area completely
        .args(rofi_options)
        .output()
        .expect("Failed to execute rofi");

    let raw_input = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if raw_input.is_empty() {
        None
    } else {
        Some(raw_input)
    }
}
