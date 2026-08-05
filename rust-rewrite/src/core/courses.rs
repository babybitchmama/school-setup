use crate::config::LessonManagerConfigFile;
use crate::rofi::select::select_from_rofi;
use crate::utils::{
    change_current_course::change_current_course, format_course_name::format_course_name,
    get_courses_in_path::get_courses_in_path, get_current_course_info::get_current_course_info,
    update_polybar_current_course::update_polybar_current_course,
};
use std::collections::HashMap;

pub fn main(config: &LessonManagerConfigFile, current_course: bool) {
    let root_dir_path = &config.root;
    let notes_dir = &config.notes_dir;
    let rofi_options = &config.rofi_options;
    let polybar_file = &config.polybar_current_course_file;

    let folder_names: Vec<String> = get_courses_in_path(root_dir_path);
    let total_courses = folder_names.len();

    let mut courses_data = Vec::with_capacity(total_courses);
    for folder in folder_names {
        let info_path = format!("{}/{}/info.yaml", root_dir_path, folder);
        let course_info = get_current_course_info(&info_path);
        courses_data.push((folder, course_info));
    }

    let global_max_len = courses_data
        .iter()
        .map(|(_, info)| info.title.len())
        .max()
        .unwrap_or(0);

    let mut course_map: HashMap<String, String> = HashMap::with_capacity(total_courses);
    let mut rofi_display_list: Vec<String> = Vec::with_capacity(total_courses);

    for (folder_name, info) in courses_data {
        let formatted_name = format_course_name(&info.title, &info.short, global_max_len);

        rofi_display_list.push(formatted_name.clone());

        course_map.insert(formatted_name, folder_name);
    }

    rofi_display_list.sort_unstable();

    let selected_formatted =
        select_from_rofi(rofi_display_list, rofi_options, "Select Course".to_string()).expect("No course selected");

    let original_course_name = course_map
        .get(&selected_formatted)
        .expect("Critical Error: Rofi returned a course not in our map");

    change_current_course(root_dir_path, original_course_name, notes_dir);
    let expanded_polybar_file = shellexpand::tilde(polybar_file);
    update_polybar_current_course(&expanded_polybar_file, original_course_name);
}
