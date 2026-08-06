use clap::{Parser, Subcommand};

mod config;
mod core;
mod dates;
mod parser;
mod rofi;
mod yaml;

use yaml::load_file;

use core::assignments;
use core::books;
// use core::calendar;
use core::courses;
// use core::inkscape;
use core::notes;

use crate::core::inkscape;
// use core::sync;

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "lesson-manager")]
#[command(about = "Managing LaTeX Lecture Notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum ThesisCommands {
    /// Manage brain dump notes (new, list)
    BrainDump { action: String },

    /// Manage meeting notes (new, list)
    Meetings { action: String },

    /// Manage section notes (new, list)
    Sections { action: String },

    /// Pull Samsung notes into corresponding folders
    Pull,

    /// Compile all notes into a single document
    Compile,

    /// Count total words across all notes
    WordCount,

    /// Show advisor summary from advisor-info.yaml
    Advisor,

    /// List stored papers and metadata from papers.bib
    Papers,
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

    Thesis {
        #[command(subcommand)]
        command: ThesisCommands,
    },

    Figures {
        action: String,

        name: Option<String>,

        #[arg(long)]
        kill: bool,
    },
}

pub fn open_in_neovim(working_dir: &Path, files: &[PathBuf], terminal: &str, editor: &str, editor_mode: &String) {
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

    let mut cmd = Command::new(terminal);
    cmd.arg(format!("--directory={}", working_dir.display()));
    cmd.arg(editor);
    cmd.args(nvim_args);

    for file in files {
        cmd.arg(file);
    }

    cmd.env("NVIM_MODE", &editor_mode).spawn().expect("Failed to open terminal and editor");
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
        Commands::Rofi {
            action,
            current_course,
        } => match action.as_str() {
            "assignments" => assignments::main(&config, *current_course),
            "books" => books::main(&config, *current_course),
            "courses" => courses::main(&config, *current_course),
            "notes" => notes::main(&config, *current_course),

            _ => println!(
                "Unknown Rofi action `{}`. Available actions: `assignments`, `books`, `courses`, `notes`.",
                action.as_str()
            ),
        },
        Commands::Thesis { command } => {
            core::thesis::main(&config, command);
        }
        Commands::Figures { action, name, kill } => {
            if *kill {
                inkscape::kill(&config);
            } else {
                inkscape::run_action(&config, action);
                println!("Running figure action: {}", action);
                if let Some(n) = name {
                    println!("Target: {}", n);
                }
            }
        }
    }
}
