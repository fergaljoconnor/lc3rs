
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::io::stdout;

pub(crate) fn getchar() -> char {
    // Largely owe this code to this Stackoverflow answer:
    // https://stackoverflow.com/questions/60130532/detect-keydown-in-rust
    enable_raw_mode().unwrap();
    let read_char = loop {
        match read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char(key),
                modifiers: KeyModifiers::NONE,
            }) => break key,
            _ => {}
        }
    };
    disable_raw_mode().unwrap();
    return read_char;
}

pub(crate) fn putchar(ch: char) {
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, Print(ch)).unwrap();
    disable_raw_mode().unwrap();
}
