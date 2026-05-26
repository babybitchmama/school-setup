use clap::{Parser, Subcommand};

mod config;
mod core;
mod rofi;
mod utils;

use utils::load_yaml_file::load_file;

use core::assignments;
use core::books;
// use core::calendar;
use core::courses;
// use core::inkscape;
use core::notes;

use crate::core::inkscape;
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
        #[arg(long)]
        current_course: bool,
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
        Commands::Rofi { action, current_course } => match action.as_str() {
            "assignments" => assignments::main(&config, *current_course),
            "books" => books::main(&config, *current_course),
            "courses" => courses::main(&config, *current_course),
            "notes" => notes::main(&config, *current_course),
            _ => println!(
                "Unknown Rofi action `{}`. Available actions: `assignments`, `books`, `courses`, `notes`.",
                action.as_str()
            ),
        },
        Commands::Figures { action, name, kill } => {
            if *kill {
                inkscape::kill(&config);
            } else {
                inkscape::run_action(&config, &action);
                println!("Running figure action: {}", action);
                if let Some(n) = name {
                    println!("Target: {}", n);
                }
            }
        }
    }
}
