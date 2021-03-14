use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use device_query::{DeviceQuery, DeviceState};

use std::io::stdout;

pub fn getchar() -> char {
    // Largely owe this code to this Stackoverflow answer:
    // https://stackoverflow.com/questions/60130532/detect-keydown-in-rust
    try_enable_raw_mode();
    let read_char = loop {
        // TODO: Replace these unwrap calls with proper error handling.
        match read().expect("Error thrown on call to crosstem::read") {
            Event::Key(KeyEvent {
                code: KeyCode::Char(key),
                modifiers: KeyModifiers::NONE,
            }) => break key,
            _ => {}
        }
    };
    try_disable_raw_mode();
    return read_char;
}

pub(crate) fn putchar(ch: char) {
    let mut stdout = stdout();
    try_enable_raw_mode();
    execute!(stdout, Print(ch)).expect("Error thrown on call to call to crossterm::execute macro");
    try_disable_raw_mode();
}

pub(crate) fn is_key_down() -> bool {
    DeviceState::new().get_keys().is_empty()
}

fn try_enable_raw_mode() {
    enable_raw_mode().expect("Error thrown on call to crossterm::terminal::enable_raw_mode");
}

fn try_disable_raw_mode() {
    disable_raw_mode().expect("Error thrown on call to crossterm::terminal::disable_raw_mode");
}
