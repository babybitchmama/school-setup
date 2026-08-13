pub mod advisor;
pub mod compile;
pub mod notes;
pub mod papers;
pub mod sync;

use crate::config::LessonManagerConfigFile;
use crate::{NoteActions, ThesisCommands};

pub fn main(config: &LessonManagerConfigFile, command: &ThesisCommands) {
    match command {
        ThesisCommands::Notes { note_type, action } => match action {
            NoteActions::New => notes::create_note(config, note_type, None),
            NoteActions::List => notes::list_notes(config, note_type),
        },

        ThesisCommands::Pull => {
            println!("Pulling Samsung notes... (Coming soon)");
        }

        ThesisCommands::Compile { targets, all } => {
            let available_types = &config.thesis_note_types;
            let mut to_compile = Vec::new();

            if *all {
                to_compile = available_types.keys().cloned().collect();
            } else if let Some(t_list) = targets {
                to_compile = t_list.clone();
            } else {
                println!("No targets provided. Use --targets <names> or --all.");
                return;
            }

            // Dispatch to the orchestrator
            crate::core::thesis::compile::compile_targets(config, &to_compile);
        }

        ThesisCommands::Advisor => {
            advisor::main(config);
        }

        ThesisCommands::Papers => {
            papers::list_papers(config);
        }
    }
}
