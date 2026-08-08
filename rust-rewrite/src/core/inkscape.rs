use crate::config::LessonManagerConfigFile;

pub fn watch_figures(_config: &LessonManagerConfigFile) {}

pub fn create_figure(_config: &LessonManagerConfigFile, _title: Option<&str>, _path: Option<&str>) {}

pub fn edit_figure(_config: &LessonManagerConfigFile, _title: Option<&str>, _path: Option<&str>) {}

pub fn manage_shortcuts(_config: &LessonManagerConfigFile) {}

pub fn kill_inkscape_processes() {}
