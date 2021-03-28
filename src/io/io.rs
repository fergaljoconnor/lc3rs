use std::io::{stdout, Write};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::error::{BoxErrors, IOResult};

pub fn getchar() -> IOResult<char> {
    // Largely owe this code to this Stackoverflow answer:
    // https://stackoverflow.com/questions/60130532/detect-keydown-in-rust
    try_enable_raw_mode()?;

    let read_char = loop {
        match read().box_error()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::NONE,
                }) => break key,

                _ => {}
            }
    };

    try_disable_raw_mode()?;

    Ok(read_char)
}

pub(crate) fn putchar(ch: char) -> IOResult<()> {
    print!("{}", ch);
    let mut stdout = stdout();
    stdout.flush().box_error()
}

fn try_enable_raw_mode() -> IOResult<()> {
    enable_raw_mode().box_error()
}

fn try_disable_raw_mode() -> IOResult<()> {
    disable_raw_mode().box_error()
}
