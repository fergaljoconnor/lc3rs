extern crate crossterm;

pub mod cli;
mod command;
mod condition_flags;
pub mod error;
pub mod io;
mod op;
pub mod plugin;
mod register;
mod trap;
#[macro_use]
mod utils;
pub mod vm;

pub use vm::VM;
pub use error::LC3Error;
