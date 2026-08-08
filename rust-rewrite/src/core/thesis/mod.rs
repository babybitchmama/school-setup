// src/core/thesis/mod.rs

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
            NoteActions::New => notes::create_note(config, note_type),
            NoteActions::List => notes::list_notes(config, note_type),
        },

        ThesisCommands::Pull => {
            println!("Pulling Samsung notes... (Coming soon)");
        }

        // ThesisCommands::Compile { targets, all } => {
        //     let available_types = &config.thesis_note_types;
        //     let mut to_compile = Vec::new();

        //     if *all {
        //         // If they pass --all, grab every note type from config.yaml
        //         to_compile = available_types.keys().cloned().collect();
        //     } else if let Some(t_list) = targets {
        //         // Check each target the user passed against the config
        //         for t in t_list {
        //             if available_types.contains_key(t) {
        //                 to_compile.push(t.clone());
        //             } else {
        //                 println!(
        //                     "Warning: '{}' is not defined in config.yaml. Skipping.",
        //                     t
        //                 );
        //             }
        //         }
        //     } else {
        //         println!("No targets provided. Running default master compile...");
        //         // Add your default compile logic here
        //     }

        //     if to_compile.is_empty() {
        //         println!("Nothing to compile.");
        //         return;
        //     }

        //     // Run the compile loop
        //     for target in to_compile {
        //         let note_config = available_types.get(&target).unwrap();
        //         println!(
        //             "Compiling '{}' from path: {} ...",
        //             target, note_config.path
        //         );

        //         // Call your actual compile functions here!
        //         // compile::run_latex_make(config, &note_config.path);
        //     }
        // }

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

            // Call the compiler
            crate::core::thesis::compile::generate_and_compile(config, &to_compile);
        }

        ThesisCommands::WordCount => {
            println!("Counting words... (Coming soon)");
        }

        ThesisCommands::Advisor => {
            advisor::main(config);
        }

        ThesisCommands::Papers => {
            papers::list_papers(config);
        }
    }
}
