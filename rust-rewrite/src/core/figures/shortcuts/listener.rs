use rdev::{grab, Event, EventType, Key};
use super::focus::is_inkscape_focused;
use super::math::MathMacroManager;
use super::styles::StyleManager;

pub fn start_listener() {
    let math_manager = MathMacroManager::new();
    let style_manager = StyleManager::load();

    println!("Starting Inkscape shortcut daemon (with event interception)...");

    let callback = move |event: Event| -> Option<Event> {
        if !is_inkscape_focused() {
            return Some(event);
        }

        match event.event_type {
            EventType::KeyPress(key) => {
                if key == Key::KeyM {
                    math_manager.trigger_latex_input();
                    return None; // Swallow key
                }

                if let Some(char_str) = &event.name {
                    if style_manager.handle_shortcut(char_str) {
                        return None; // Swallow key so Inkscape never sees 't', 'r', etc.
                    }
                }
            }
            EventType::KeyRelease(key) => {
                if key == Key::KeyM {
                    return None;
                }
            }
            _ => {}
        }

        Some(event)
    };

    if let Err(error) = grab(callback) {
        eprintln!("❌ Error in event grabber: {:?}", error);
    }
}
