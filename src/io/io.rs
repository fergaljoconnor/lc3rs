use std::io::{stdout, Write};

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::error::{BoxErrors, LC3Result};

pub fn getchar() -> LC3Result<char> {
    // Largely owe this code to this Stackoverflow answer:
    // https://stackoverflow.com/questions/60130532/detect-keydown-in-rust
    try_enable_raw_mode()?;

    let read_char = loop {
        match read().map_io_error()? {
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

pub(crate) fn putchar(ch: char) -> LC3Result<()> {
    print!("{}", ch);
    let mut stdout = stdout();
    stdout.flush().map_io_error()
}

fn try_enable_raw_mode() -> LC3Result<()> {
    enable_raw_mode().map_io_error()
}

fn try_disable_raw_mode() -> LC3Result<()> {
    disable_raw_mode().map_io_error()
}
