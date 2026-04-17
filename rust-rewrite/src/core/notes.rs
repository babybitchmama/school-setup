use crate::core::courses::get_current_course_info;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;
use crate::utils::parser::{get_week, pad_number, parse_range_string};
use chrono::NaiveDateTime;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Note {
    pub file_path: PathBuf,
    pub number: Option<u32>,
    pub date: Option<NaiveDateTime>,
    pub title: Option<String>,
    pub week: Option<i32>,
    pub display_number: Option<u32>,
}

impl Note {
    pub fn new(file_path: PathBuf, date_format: &str, start_date_str: &str) -> Self {
        let mut note = Note {
            file_path,
            number: None,
            date: None,
            title: None,
            week: None,
            display_number: None,
        };
        note.parse_note_file(date_format, start_date_str);
        note
    }

    fn parse_note_file(&mut self, date_format: &str, start_date_str: &str) {
        let Ok(content) = fs::read_to_string(&self.file_path) else {
            return;
        };

        let re = Regex::new(r"\\lecture\{(\d+)\}\{(.+?)\}\{(.+?)\}").unwrap();

        for line in content.lines() {
            if let Some(caps) = re.captures(line) {
                self.number = caps.get(1).unwrap().as_str().parse::<u32>().ok();
                self.display_number = self.number.map(|n| n.to_string().parse::<u32>().unwrap_or(n));

                if let Ok(date) =
                    NaiveDateTime::parse_from_str(caps.get(2).unwrap().as_str(), date_format)
                {
                    self.date = Some(date);

                    if let Ok(start_date) =
                        NaiveDateTime::parse_from_str(start_date_str, date_format)
                    {
                        self.week =
                            Some((get_week(date) as i32) - (get_week(start_date) as i32) + 1);
                    }
                }

                self.title = Some(caps.get(3).unwrap().as_str().to_string());
                break;
            }
        }
    }

    pub fn edit(&self, current_course_dir: &str, editor: &str) {
        let listen_location = "/tmp/nvim.pipe";
        let mut nvim_args = Vec::new();

        if Path::new(listen_location).exists() {
            nvim_args.push("--server");
            nvim_args.push(listen_location);
            nvim_args.push("--remote-tab");
        } else {
            nvim_args.push("--listen");
            nvim_args.push(listen_location);
        }

        let num_str = pad_number(self.number.unwrap_or(0));
        let note_file = format!("lectures/lec-{}.tex", num_str);

        Command::new("kitty")
            .arg(format!("--directory={}", current_course_dir))
            .arg(editor)
            .args(nvim_args)
            .arg(&note_file)
            .spawn()
            .expect("Failed to open Kitty/Neovim");
    }

    pub fn title_len(&self) -> usize {
        self.title.as_ref().map(|t| t.len()).unwrap_or(0)
    }

    pub fn format_display(&self, date_format: &str, max_title_len: usize) -> String {
        let num_str = std::fmt::format(format_args!(" {:2}", self.number.unwrap_or(0)));
        let title = self.title.clone().unwrap_or_else(|| "Untitled".to_string());

        let date_str = self
            .date
            .map(|d| d.format(date_format).to_string())
            .unwrap_or_else(|| "Unknown Date".to_string());

        let week = self.week.unwrap_or(0);

        let padding_needed = max_title_len.saturating_sub(title.len()) + 4;
        let padding = "\u{00A0}".repeat(padding_needed);

        let display_str = format!(
            "{num}. <b>{title}</b>{pad}<small>{date} (Week: {week})</small>",
            num = num_str,
            title = title,
            pad = padding,
            date = date_str,
            week = week
        );

        display_str
    }
}

pub struct Notes {
    pub root: PathBuf,
    pub master_file: PathBuf,
    pub notes_path: PathBuf,
    pub items: Vec<Note>,
}

impl Notes {
    pub fn new(course_root: &str, date_format: &str, start_date_str: &str) -> Self {
        let root = PathBuf::from(course_root);
        let mut notes = Notes {
            master_file: root.join("master.tex"),
            notes_path: root.join("lectures"),
            root,
            items: Vec::new(),
        };
        notes.read_files(date_format, start_date_str);
        notes
    }

    fn read_files(&mut self, date_format: &str, start_date_str: &str) {
        let pattern = format!("{}/lec-*.tex", self.notes_path.display());
        let mut parsed_notes = Vec::new();

        if let Ok(entries) = glob::glob(&pattern) {
            for entry in entries.flatten() {
                parsed_notes.push(Note::new(entry, date_format, start_date_str));
            }
        }

        parsed_notes.retain(|n| n.number.is_some());
        parsed_notes.sort_by_key(|n| n.number.unwrap());

        self.items = parsed_notes;
    }

    pub fn include_lecture(&self, lecture_number: u32) -> String {
        let num_str = pad_number(lecture_number);
        format!("\\input{{./lectures/lec-{}.tex}}\n", num_str)
    }

    fn filter_body(&self, target_numbers: &[u32]) -> String {
        let Ok(content) = fs::read_to_string(&self.master_file) else {
            return String::new();
        };

        let mut filtered_body = String::new();
        let mut in_notes_block = false;
        let mut lowest = 0;
        let mut highest = 0;

        let start_re = Regex::new(r"% notes start (\d+)-(\d+)").unwrap();

        for line in content.lines() {
            let stripped = line.trim();

            if stripped.starts_with(r"\chapter") {
                filtered_body.push_str(line);
                filtered_body.push('\n');
            } else if stripped.contains("% notes start") {
                if let Some(caps) = start_re.captures(stripped) {
                    lowest = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    highest = caps.get(2).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    in_notes_block = true;
                }
                filtered_body.push_str(line);
                filtered_body.push('\n');

                for &num in target_numbers {
                    if num >= lowest && num <= highest {
                        filtered_body.push_str(&self.include_lecture(num));
                    }
                }
            } else if stripped.contains("% notes end") {
                in_notes_block = false;
                filtered_body.push_str(line);
                filtered_body.push('\n');
            } else if !in_notes_block {
                filtered_body.push_str(line);
                filtered_body.push('\n');
            }
        }

        filtered_body
    }

    pub fn update_notes_in_master(&self, target_numbers: &[u32]) {
        let body = self.filter_body(target_numbers);
        fs::write(&self.master_file, body).expect("Failed to write to master.tex");
    }

    pub fn compile_master(&self) {
        Command::new("make")
            .current_dir(&self.root)
            .status()
            .expect("Failed to execute make command");
    }
}

pub fn main(
    root_dir_path: &str,
    notes_dir: &str,
    rofi_options: &[String],
    date_format: &str,
    polybar_file: &str,
) {
    let expanded_notes = shellexpand::tilde(notes_dir);
    let current_course_dir = format!("{}/current-course", expanded_notes);

    let info_path = format!("{}/info.yaml", current_course_dir);
    let course_info = get_current_course_info(&info_path);

    let my_notes = Notes::new(&current_course_dir, date_format, &course_info.start_date);

    if my_notes.items.is_empty() {
        message("No notes found for this course.", "info", rofi_options);
        return;
    }

    let mut sorted_notes = my_notes.items.clone();
    sorted_notes.sort_by(|a, b| b.number.unwrap_or(0).cmp(&a.number.unwrap_or(0)));

    let max_title_len = sorted_notes
        .iter()
        .map(|n| n.title_len())
        .max()
        .unwrap_or(0);

    let mut rofi_display_list = Vec::with_capacity(sorted_notes.len());
    let mut note_map = HashMap::with_capacity(sorted_notes.len());

    for note in sorted_notes {
        let display_str = note.format_display(date_format, max_title_len);
        rofi_display_list.push(display_str.clone());
        note_map.insert(display_str, note);
    }

    if let Some(selected) = select_from_rofi(rofi_display_list, rofi_options) {
        if let Some(selected_note) = note_map.get(&selected) {
            println!(
                "Opening Lecture {} in Neovim...",
                selected_note.number.unwrap_or(0)
            );
            selected_note.edit(&current_course_dir, "nvim");
        }
    }
}
