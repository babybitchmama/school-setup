pub mod advisor;
pub mod meetings;
pub mod notes;
pub mod papers;
pub mod sections;
pub mod sync;

// Import both enums from your main crate
use crate::config::LessonManagerConfigFile;
use crate::{ThesisActions, ThesisCommands};

pub fn main(config: &LessonManagerConfigFile, command: &ThesisCommands) {
    match command {
        ThesisCommands::BrainDump { action } => match action {
            ThesisActions::New => notes::new_brain_dump(config),
            ThesisActions::List => notes::list_brain_dump_files(config),
        },

        ThesisCommands::Meetings { action } => match action {
            ThesisActions::New => meetings::new_meeting(config),
            ThesisActions::List => meetings::list_meetings(config),
        },

        ThesisCommands::Sections { action } => match action {
            ThesisActions::New => sections::new_section(config),
            ThesisActions::List => sections::list_section_notes(config),
        },

        ThesisCommands::Pull => {
            println!("Pulling Samsung notes... (Coming soon)");
            // sync::pull_notes(config);
        }

        ThesisCommands::Compile {
            brain_dumps,
            meeting_notes,
            sections,
        } => {
            if !brain_dumps && !meeting_notes && !sections {
                println!("No specific targets passed. Running default master compile...");
                // compile::run_master(config);
            }

            if *brain_dumps {
                println!("Compiling brain dumps...");
                // compile::run_brain_dumps(config);
            }
            if *meeting_notes {
                println!("Compiling meeting notes...");
                // compile::run_meeting_notes(config);
            }
            if *sections {
                println!("Compiling sections...");
                // compile::run_sections(config);
            }
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
