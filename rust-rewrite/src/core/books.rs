use crate::config::LessonManagerConfigFile;
use crate::utils::get_files::get_content;
use prettytable::{Table, row};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub enum Solutions {
    SingleFile(PathBuf),
    MultipleFiles(HashMap<String, PathBuf>),
    None,
}

pub struct BookData {
    pub path: PathBuf,
    pub solutions: Solutions,
}

pub struct Book {
    expanded_course_path: PathBuf,
    expanded_book_directory: PathBuf,
    expanded_solutions_directory: PathBuf,
    master_file_path: PathBuf,
    name: String,
    display_name: String,
    solution_files: Vec<String>,
    data: HashMap<String, BookData>,
}

fn to_title_case(s: &str) -> String {
    let small_words = [
        "and", "or", "of", "the", "a", "an", "in", "on", "at", "to", "for", "with", "but", "nor",
    ];

    s.split('-')
        .enumerate()
        .map(|(i, word)| {
            if i == 0 || !small_words.contains(&word) {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_books(data: &HashMap<String, BookData>) {
    let mut table = Table::new();
    table.add_row(row!["Name", "Path", "Solutions"]);

    for (name, book_data) in data {
        let solutions = match &book_data.solutions {
            Solutions::SingleFile(path) => path.display().to_string(),
            Solutions::MultipleFiles(map) => map.keys().cloned().collect::<Vec<_>>().join(", "),
            Solutions::None => "None".to_string(),
        };

        table.add_row(row![name, book_data.path.display(), solutions]);
    }

    table.printstd();
}

impl Book {
    pub fn new(config: &LessonManagerConfigFile, course: &str) -> Self {
        let books_folder = &config.books_folder;
        let root = &config.root;
        let expanded_root = shellexpand::tilde(root);
        let book_solution_folder_relative =
            format!("{}/{}", config.books_folder, config.book_solution_folder);

        let course_path = format!("{}/{}", expanded_root, course);
        let expanded_book_directory = format!("{}/{}", course_path, book_solution_folder_relative);
        let expanded_solutions_directory =
            format!("{}/{}", course_path, book_solution_folder_relative);

        let (mut master_file_path, name) = Self::get_master_file_path(&course_path, books_folder);
        master_file_path = shellexpand::tilde(&master_file_path).into_owned();

        Book {
            expanded_course_path: PathBuf::from(course_path.clone()),
            expanded_book_directory: PathBuf::from(expanded_book_directory),
            expanded_solutions_directory: PathBuf::from(expanded_solutions_directory),
            master_file_path: PathBuf::from(master_file_path),
            name,
            display_name: String::new(),
            solution_files: Vec::new(),
            data: HashMap::new(),
        }
    }

    fn get_master_file_path(course_path: &str, books_folder: &str) -> (String, String) {
        let books_directory = format!("{}/{}", course_path, books_folder);
        let books_directory_content = get_content(&books_directory);
        let mut data: HashMap<String, BookData> = HashMap::new();

        for file in books_directory_content {
            let file_path = PathBuf::from(&file);

            if file_path.is_file() {
                let stem = file_path.file_stem().unwrap().to_str().unwrap();
                let display_name = to_title_case(stem);

                data.insert(
                    display_name,
                    BookData {
                        path: file_path,
                        solutions: Solutions::None,
                    },
                );
            } else if file_path.is_dir() {
                let master_file = file_path.join("master.pdf");
                let solutions_pdf = file_path.join("solutions.pdf");
                let solutions_dir = file_path.join("solutions");

                if master_file.exists() {
                    let stem = file_path.file_name().unwrap().to_str().unwrap();
                    let display_name = to_title_case(stem);

                    let solutions = if solutions_pdf.exists() {
                        Solutions::SingleFile(solutions_pdf)
                    } else if solutions_dir.exists() {
                        let mut map = HashMap::new();
                        for entry in fs::read_dir(&solutions_dir)
                            .unwrap()
                            .filter_map(|e: Result<_, _>| e.ok())
                        {
                            let solution_path: PathBuf = entry.path();
                            let solution_name = to_title_case(
                                solution_path.file_stem().unwrap().to_str().unwrap(),
                            );
                            map.insert(solution_name, solution_path);
                        }
                        Solutions::MultipleFiles(map)
                    } else {
                        Solutions::None
                    };

                    data.insert(
                        display_name,
                        BookData {
                            path: master_file,
                            solutions,
                        },
                    );
                }
            }
        }

        print_books(&data);

        (String::new(), String::new())
    }
}

pub fn main(config: &LessonManagerConfigFile, current_course: bool) {
    let book = Book::new(&config, "mth-445");
}
