use crate::config::AdvisorInfo;
use crate::rofi::message::plain_message;

pub fn main(config: &crate::config::LessonManagerConfigFile) {
    let advisor_path = format!("{}/{}", shellexpand::tilde(&config.thesis_dir), &config.thesis_advisor_info_file);
    let advisor: AdvisorInfo = crate::yaml::load_file(&advisor_path)
        .expect("Failed to load advisor.yaml");

    let display = advisor.fields
        .iter()
        .map(|(key, value): (&String, &serde_yaml::Value)| {
            let formatted_key = key
                .replace("_", " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");

            match value {
                serde_yaml::Value::Mapping(map) => {
                    let nested = map.iter()
                        .map(|(k, v)| format!("  {}: {}", k.as_str().unwrap_or(""), v.as_str().unwrap_or("")))
                        .collect::<Vec<String>>()
                        .join("\n");
                    format!("<b>{}:</b>\n{}", formatted_key, nested)
                }
                _ => format!("<b>{}:</b> {}", formatted_key, value.as_str().unwrap_or("")),
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    let display = format!("{}\n", display);

    plain_message(&display, &config.rofi_options);
}
