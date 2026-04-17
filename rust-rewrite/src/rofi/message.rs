pub fn message(message: &str, message_type: &str, rofi_arguments: &[String]) {
    let mut color = "green";
    if message_type == "error" {
        color = "red";
    } else if message_type == "warning" {
        color = "orange";
    }

    let full_message = format!("<span color='{}'><b>Info:</b> {}</span>", color, message);

    let _ = std::process::Command::new("rofi")
        .args(rofi_arguments)
        .arg("-markup")
        .arg("-e")
        .arg(full_message)
        .spawn()
        .expect("Failed to start rofi");
}
