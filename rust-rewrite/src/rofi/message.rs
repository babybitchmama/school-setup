pub fn plain_message(message: &str, rofi_arguments: &[String]) {
    let _ = std::process::Command::new("rofi")
        .args(rofi_arguments)
        .arg("-markup")
        .arg("-e")
        .arg(message)
        .spawn()
        .expect("Failed to start rofi");
}


pub fn message(message: &str, message_type: &str, rofi_arguments: &[String], prompt: Option<&str>) {
    let prompt = prompt.unwrap_or("Info: ");
    let prompt = if prompt.is_empty() {
        "".to_string()
    } else {
        format!("{}: ", prompt)
    };

    let mut color = "green";
    if message_type == "error" {
        color = "red";
    } else if message_type == "warning" {
        color = "orange";
    }

    let full_message = format!("<span color='{}'><b>{}</b>{}</span>", color, prompt, message);

    plain_message(&full_message, rofi_arguments);
}
