use std::collections::HashMap;
use crate::config::LessonManagerConfigFile;
use crate::rofi::select::select_from_rofi;
use crate::utils::{
    format_course_name::format_course_name, get_courses_in_path::get_courses_in_path,
};

pub fn select_course(config: &LessonManagerConfigFile) -> Option<String> {
    let folder_names = get_courses_in_path(&config.root);

    if folder_names.is_empty() {
        return None;
    }

    let mut courses_data = Vec::new();
    for folder in &folder_names {
        let info_path = format!("{}/{}/info.yaml", config.root, folder);
        let expanded = shellexpand::tilde(&info_path);
        if let Ok(info) = crate::utils::load_yaml_file::load_file::<crate::config::CourseYamlFile>(expanded.as_ref()) {
            courses_data.push((folder.clone(), info));
        }
    }

    let global_max_len = courses_data
        .iter()
        .map(|(_, info)| info.title.len())
        .max()
        .unwrap_or(0);

    let mut rofi_display_list = Vec::new();
    let mut course_map: HashMap<String, String> = HashMap::new();

    for (folder_name, info) in courses_data {
        let formatted = format_course_name(&info.title, &info.short, global_max_len);
        rofi_display_list.push(formatted.clone());
        course_map.insert(formatted, folder_name);
    }

    rofi_display_list.sort_unstable();

    let selected = select_from_rofi(rofi_display_list, &config.rofi_options, "Select Course".to_string())?;

    course_map.get(&selected).cloned()
}
