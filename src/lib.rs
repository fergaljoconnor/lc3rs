#[macro_use]
extern crate crossterm;

pub mod cli;
mod command;
mod condition_flags;
pub mod io;
mod op;
mod register;
mod trap;
#[macro_use]
mod utils;
pub mod vm;
