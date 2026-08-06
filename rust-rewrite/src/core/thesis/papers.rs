use crate::config::LessonManagerConfigFile;
use crate::rofi::message::message;
use crate::rofi::select::select_from_rofi;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default, Debug, Clone)]
struct BibEntry {
    key: String,
    title: String,
    author: String,
    year: String,
}

pub fn list_papers(config: &LessonManagerConfigFile) {
    let thesis_dir = &config.thesis_dir;
    let pdf_viewer = &config.pdf_viewer;
    let thesis_path = PathBuf::from(shellexpand::tilde(thesis_dir).to_string());
    let papers_dir = thesis_path.join("papers");
    let bib_path = papers_dir.join("papers.bib");

    if !bib_path.exists() {
        message(
            "papers.bib not found in the papers directory.",
            "error",
            &config.rofi_options,
            None,
        );
        return;
    }

    let bib_content = match fs::read_to_string(&bib_path) {
        Ok(content) => content,
        Err(_) => {
            message(
                "Failed to read papers.bib",
                "error",
                &config.rofi_options,
                None,
            );
            return;
        }
    };

    let mut papers = Vec::new();
    let mut current_paper = BibEntry::default();
    let mut in_entry = false;

    for line in bib_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('@') {
            if in_entry && !current_paper.key.is_empty() {
                papers.push(current_paper.clone());
            }
            current_paper = BibEntry::default();
            in_entry = true;

            // FIXED: Search the whole string for the comma to get the absolute index
            if let Some(start) = trimmed.find('{') {
                if let Some(end) = trimmed.find(',') {
                    if end > start {
                        // Added .trim() to safely remove any rogue spaces
                        current_paper.key = trimmed[start + 1..end].trim().to_string();
                    }
                }
            }
        } else if in_entry {
            let lower_line = trimmed.to_lowercase();
            if lower_line.starts_with("title") {
                current_paper.title = extract_bib_value(trimmed);
            } else if lower_line.starts_with("author") {
                current_paper.author = extract_bib_value(trimmed);
            } else if lower_line.starts_with("year") {
                current_paper.year = extract_bib_value(trimmed);
            }
        }
    }

    if in_entry && !current_paper.key.is_empty() {
        papers.push(current_paper);
    }

    if papers.is_empty() {
        message(
            "No entries found in papers.bib",
            "info",
            &config.rofi_options,
            None,
        );
        return;
    }

    let mut display_list = Vec::new();
    let mut path_map = HashMap::new();

    for paper in papers {
        display_paper_info(&paper, &mut display_list, &mut path_map, &papers_dir);
    }

    if let Some(selected) = select_from_rofi(
        display_list,
        &config.rofi_options,
        "Select a paper:".to_string(),
    ) {
        if let Some(pdf_path) = path_map.get(&selected) {
            if pdf_path.exists() {
                Command::new(pdf_viewer)
                    .arg(pdf_path)
                    .spawn()
                    .expect("Failed to launch PDF viewer");
            } else {
                let file_name = pdf_path.file_name().unwrap_or_default().to_string_lossy();
                message(
                    &format!("PDF not found: {}", file_name),
                    "error",
                    &config.rofi_options,
                    None,
                );
            }
        }
    }
}

fn display_paper_info(paper: &BibEntry, display_list: &mut Vec<String>, path_map: &mut HashMap<String, PathBuf>, papers_dir: &PathBuf) {
    let display_title = if paper.title.is_empty() {
        &paper.key
    } else {
        &paper.title
    };
    let author_str = if paper.author.is_empty() {
        String::new()
    } else {
        format!(" by {}", paper.author)
    };
    let year_str = if paper.year.is_empty() {
        String::new()
    } else {
        format!(" ({})", paper.year)
    };

    let display_str = format!(
        "<b>{}</b><i><small>{}{}</small></i>",
        display_title, author_str, year_str
    );

    display_list.push(display_str.clone());

    let new_key = if paper.key.len() > 4 {
        &paper.key[..paper.key.len() - 4]
    } else {
        &paper.key
    };

    let pdf_path = papers_dir.join(format!("{}.pdf", new_key));
    path_map.insert(display_str, pdf_path);
}

// Helper to pull text out of BibTeX braces `{...}` or quotes `"..."`
fn extract_bib_value(line: &str) -> String {
    if let Some(start) = line.find('{') {
        if let Some(end) = line.rfind('}') {
            if end > start {
                return line[start + 1..end].trim().to_string();
            }
        }
    }
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if end > start {
                return line[start + 1..end].trim().to_string();
            }
        }
    }
    String::new()
}
