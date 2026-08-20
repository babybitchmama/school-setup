use clap::Parser;

mod commands;
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
use core::notes;

use commands::{Commands, RofiCommands, NoteActions, ThesisCommands, FigureCommands};

// use core::sync;

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "lesson-manager")]
#[command(about = "Managing LaTeX Lecture Notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

pub fn open_in_neovim(
    working_dir: &Path,
    files: &[PathBuf],
    terminal: &str,
    editor: &str,
    editor_mode: &String,
    terminal_class_name: Option<&String>,
) {
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

    if let Some(class) = terminal_class_name {
        cmd.arg("--class").arg(class);
    }

    cmd.arg(format!("--directory={}", working_dir.display()));
    cmd.arg(editor);
    cmd.args(nvim_args);

    for file in files {
        cmd.arg(file);
    }

    // Explicitly inherit the current environment (DISPLAY, DBUS, etc.) so Alacritty can talk to X11/Wayland
    cmd.env("NVIM_MODE", editor_mode);
    cmd.envs(std::env::vars());

    let _ = cmd.spawn().expect("Failed to open terminal and editor").wait();
}

fn handle_rofi_command(
    command: &RofiCommands,
    config: &config::LessonManagerConfigFile,
    select_course: bool,
) {
    match command {
        RofiCommands::Assignments => {
            assignments::main(config, select_course);
        }
        RofiCommands::Books => {
            books::main(config, select_course);
        }
        RofiCommands::Courses => {
            courses::main(config, select_course);
        }
        RofiCommands::Notes => {
            notes::main(config, select_course);
        }
    }
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
            command,
            select_course,
        } => {
            handle_rofi_command(command, &config, *select_course);
        }
        Commands::Thesis { command } => {
            core::thesis::main(&config, command);
        }
        Commands::Figures { command } => {
            core::figures::main(&config, command);
        }
    }
}
