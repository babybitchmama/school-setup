pub mod advisor;
pub mod meetings;
pub mod notes;
pub mod papers;
pub mod sections;
pub mod sync;

use crate::ThesisCommands;
use crate::config::LessonManagerConfigFile;

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn main(config: &LessonManagerConfigFile, command: &ThesisCommands) {
    match command {
        ThesisCommands::BrainDump { action } => match action.as_str() {
            "new" => notes::new_brain_dump(config),
            "list" => notes::list_brain_dump_files(config),
            _ => println!("Unknown brain-dump action: '{}'. Try 'new' or 'list'.", action),
        },

        ThesisCommands::Meetings { action } => match action.as_str() {
            "new" => meetings::new_meeting(config),
            "list" => meetings::list_meetings(config),
            _ => println!("Unknown meetings action: '{}'. Try 'new' or 'list'.", action),
        },

        ThesisCommands::Sections { action } => match action.as_str() {
            "new" => sections::new_section(config),
            "list" => sections::list_section_notes(config),
            _ => println!("Unknown sections action: '{}'. Try 'new' or 'list'.", action),
        },

        ThesisCommands::Pull => {
            println!("Pulling Samsung notes... (Coming soon)");
            // sync::pull_notes(config);
        }

        ThesisCommands::Compile => {
            println!("Compiling thesis... (Coming soon)");
            // compile::run_compile(config);
        }

        ThesisCommands::WordCount => {
            println!("Counting words... (Coming soon)");
            // compile::word_count(config);
        }

        ThesisCommands::Advisor => {
            advisor::main(config);
        }

        ThesisCommands::Papers => {
            papers::list_papers(config);
        }
    }
}
