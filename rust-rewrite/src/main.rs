use clap::{Parser, Subcommand};

mod config;
mod core;
mod rofi;
mod utils;

use utils::load_yaml_file::load_file;

// use core::assignments;
// use core::books;
// use core::calendar;
use core::courses;
// use core::inkscape;
use core::notes;
// use core::sync;

#[derive(Parser)]
#[command(name = "lesson-manager")]
#[command(about = "Managing LaTeX Lecture Notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Calendar,

    InitCourses,

    Rofi {
        action: String,
    },

    BenchmarkRofi {
        ms: u64,
    },

    Figures {
        action: String,

        name: Option<String>,

        #[arg(long)]
        kill: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let config_path = "~/.config/lesson-manager/config.yaml";
    let expanded_config_file_path = shellexpand::tilde(config_path);
    let config: config::LessonManagerConfigFile = load_file(&expanded_config_file_path).unwrap();

    match &cli.command {
        Commands::Calendar => {
            println!("Hooking into calendar...");
        }
        Commands::InitCourses => {
            println!("Initializing course directories...");
        }
        Commands::Rofi { action } => {
            match action.as_str() {
                "assignments" => println!("Opening Rofi for assignments..."),
                "books" => println!("Opening Rofi for books..."),
                "courses" => courses::main(&config.root, &config.notes_dir, &config.rofi_options, &config.polybar_current_course_file),
                "notes" => notes::main(&config.notes_dir, &config.rofi_options, &config.date_format),
                _ => println!("Unknown Rofi action. Available actions: `assignments`, `books`, `courses`, `notes`."),
            }
        }
        Commands::Figures { action, name, kill } => {
            if *kill {
                println!("Killing the {} process...", action);
            } else {
                println!("Running figure action: {}", action);
                if let Some(n) = name {
                    println!("Target: {}", n);
                }
            }
        }
        Commands::BenchmarkRofi { ms } => {
            use std::process::{Command, Stdio};
            use std::io::Write;
            use std::time::Duration;
            use std::thread;

            println!("Testing Time-to-Glass with a {}ms timeout...", ms);

            let mut child = Command::new("rofi")
                .arg("-dmenu")
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to start rofi");

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(b"Test 1\nTest 2\nTest 3").unwrap();
            }

            thread::sleep(Duration::from_millis(*ms));

            let _ = child.kill();
            let _ = child.wait();

            println!("Did you see it blink?");
        }
    }
}
