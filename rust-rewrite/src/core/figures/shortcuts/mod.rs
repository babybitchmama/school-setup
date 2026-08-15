mod focus;
mod listener;
mod math;
mod styles;

use listener::start_listener;

/// Entry point for the `lm figures shortcuts` command
pub fn execute_shortcuts() {
    println!("Starting Inkscape shortcut daemon...");

    start_listener();
}
