use std::fs;

pub fn get_content(path: &str) -> Vec<String> {
    let mut content = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(path_str) = path.to_str() {
                    content.push(path_str.to_string());
                }
            }
        }
    }

    content
}
