use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::io:: {Write, stdout};

// TODO: Something in these functions is creating too many clients and
// overloading the terminal, causing random crashes during is_key_down.

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
    print!("{}", ch);
    let mut stdout = stdout();
    stdout.flush().expect("Error thrown during stdout flush");
}

fn try_enable_raw_mode() {
    enable_raw_mode().expect("Error thrown on call to crossterm::terminal::enable_raw_mode");
}

fn try_disable_raw_mode() {
    disable_raw_mode().expect("Error thrown on call to crossterm::terminal::disable_raw_mode");
}
