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
use core::inkscape;
use core::notes;

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
pub enum RofiCommands {
    Assignments,
    Books,
    Courses,
    Notes,
}

#[derive(Subcommand, Debug)]
pub enum ThesisCompileOptions {
    BrainDumps,
    MeetingNotes,
    SectionNotes,
}

#[derive(Subcommand, Debug)]
pub enum ThesisActions {
    New,
    List,
}

#[derive(Subcommand, Debug)]
pub enum ThesisCommands {
    /// Manage brain dump notes (new, list)
    BrainDump {
        #[command(subcommand)]
        action: ThesisActions,
    },

    /// Manage meeting notes (new, list)
    Meetings {
        #[command(subcommand)]
        action: ThesisActions,
    },

    /// Manage section notes (new, list)
    Sections {
        #[command(subcommand)]
        action: ThesisActions,
    },

    /// Compile all notes into a single document
    Compile {
        /// Compile brain dump notes
        #[arg(long)]
        brain_dumps: bool,

        /// Compile meeting notes
        #[arg(long)]
        meeting_notes: bool,

        /// Compile section notes
        #[arg(long)]
        sections: bool,
    },

    /// Pull Samsung notes into corresponding folders
    Pull,

    /// Count total words across all notes
    WordCount,

    /// Show advisor summary from advisor-info.yaml
    Advisor,

    /// List stored papers and metadata from papers.bib
    Papers,
}

#[derive(Subcommand, Debug)]
pub enum FigureCommands {
    /// Watch for figures
    Watch,

    /// Create a figure
    Create {
        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        path: Option<String>,
    },

    /// Edit a figure
    Edit {
        /// Name of the figure to edit is optional
        #[arg(long)]
        title: Option<String>,

        /// Or you could specify the path to the figure files, and all the figures will be displayed via rofi
        #[arg(long)]
        path: Option<String>,
    },

    /// Shortcut manager, start process to monitor keystrokes
    Shortcuts,

    /// Kill all running Inkscape processes
    Kill,
}

#[derive(Subcommand)]
enum Commands {
    Calendar,

    InitCourses,

    Rofi {
        #[command(subcommand)]
        command: RofiCommands,
        #[arg(long)]
        current_course: bool,
    },

    Thesis {
        #[command(subcommand)]
        command: ThesisCommands,
    },

    Figures {
        #[command(subcommand)]
        command: FigureCommands,
    },
}

pub fn open_in_neovim(
    working_dir: &Path,
    files: &[PathBuf],
    terminal: &str,
    editor: &str,
    editor_mode: &String,
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
    cmd.arg(format!("--directory={}", working_dir.display()));
    cmd.arg(editor);
    cmd.args(nvim_args);

    for file in files {
        cmd.arg(file);
    }

    cmd.env("NVIM_MODE", editor_mode)
        .spawn()
        .expect("Failed to open terminal and editor");
}

fn handle_rofi_command(
    command: &RofiCommands,
    config: &config::LessonManagerConfigFile,
    current_course: bool,
) {
    match command {
        RofiCommands::Assignments => {
            assignments::main(config, current_course);
        }
        RofiCommands::Books => {
            books::main(config, current_course);
        }
        RofiCommands::Courses => {
            courses::main(config, current_course);
        }
        RofiCommands::Notes => {
            notes::main(config, current_course);
        }
    }
}

fn handle_figure_command(command: &FigureCommands, config: &config::LessonManagerConfigFile) {
    match command {
        FigureCommands::Watch => {
            inkscape::watch_figures(config);
        }
        FigureCommands::Create { title, path } => {
            inkscape::create_figure(config, title.as_deref(), path.as_deref());
        }
        FigureCommands::Edit { title, path } => {
            inkscape::edit_figure(config, title.as_deref(), path.as_deref());
        }
        FigureCommands::Shortcuts => {
            inkscape::manage_shortcuts(config);
        }
        FigureCommands::Kill => {
            inkscape::kill_inkscape_processes();
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
            current_course,
        } => {
            handle_rofi_command(command, &config, *current_course);
        }
        Commands::Thesis { command } => {
            core::thesis::main(&config, command);
        }
        Commands::Figures { command } => {
            handle_figure_command(command, &config);
        }
    }
}
