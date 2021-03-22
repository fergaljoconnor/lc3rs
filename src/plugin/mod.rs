mod plugin;
pub mod debuglogger;
#[cfg(test)]
mod test;

pub use plugin::{Plugin, Event};
