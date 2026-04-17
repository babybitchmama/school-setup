pub fn message(message: &str, message_type: &str, rofi_arguments: &[String]) {
    let mut color = "green";
    if message_type == "error" {
        color = "red";
    } else if message_type == "warning" {
        color = "orange";
    }

    let full_message = format!("<span color='{}'><b>Info:</b> {}</span>", color, message);

    let _ = std::process::Command::new("rofi")
        .args(rofi_arguments) // 1. Pass all standard config flags first
        .arg("-markup")       // 2. Explicitly enable markup for the popup box
        .arg("-e")            // 3. Declare the popup flag
        .arg(full_message)    // 4. The message MUST immediately follow -e
        .spawn()
        .expect("Failed to start rofi");
}
