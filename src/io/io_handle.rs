#[cfg(test)]
use std::cell::RefCell;

use device_query::{DeviceQuery, DeviceState};

use super::io::{getchar, putchar};
use crate::error::LC3Result;

#[cfg(test)]
use crate::error::{BoxErrors, LC3Error};

// TODO: Maybe the dependency should be flipped here, so this trait should
// be part of the VM module. It's the VM's needs that actually determine
// what this interface should do.
pub trait IOHandle {
    fn getchar(&self) -> LC3Result<char>;
    fn putchar(&self, ch: char) -> LC3Result<()>;
    fn is_key_down(&self) -> LC3Result<bool>;
}

pub struct RealIOHandle {
    device_state: DeviceState,
}

impl RealIOHandle {
    pub fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
        }
    }
}

impl IOHandle for RealIOHandle {
    fn getchar(&self) -> LC3Result<char> {
        getchar()
    }

    fn putchar(&self, ch: char) -> LC3Result<()> {
        putchar(ch)
    }

    fn is_key_down(&self) -> LC3Result<bool> {
        Ok(self.device_state.get_keys().is_empty())
    }
}

#[cfg(test)]
pub(crate) struct TestIOHandle {
    key_presses: RefCell<Vec<char>>,
    outputs: RefCell<Vec<char>>,
    keydown_values: RefCell<Vec<bool>>,
}

#[cfg(test)]
impl TestIOHandle {
    pub(crate) fn new() -> Self {
        Self {
            key_presses: RefCell::new(Vec::new()),
            outputs: RefCell::new(Vec::new()),
            keydown_values: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn add_key_press(&mut self, ch: char) {
        self.key_presses.borrow_mut().push(ch);
    }

    pub(crate) fn add_keydown_response(&mut self, val: bool) {
        self.keydown_values.borrow_mut().push(val)
    }

    pub(crate) fn get_test_outputs(&self) -> Vec<char> {
        self.outputs.borrow().clone()
    }
}

#[cfg(test)]
impl IOHandle for TestIOHandle {
    fn getchar(&self) -> LC3Result<char> {
        self.key_presses
            .borrow_mut()
            .pop()
            .ok_or(LC3Error::Other(
                "Attempted to call getchar on empty key presses vector".to_string(),
            ))
    }

    fn putchar(&self, ch: char) -> LC3Result<()> {
        self.outputs.borrow_mut().push(ch);
        Ok(())
    }

    fn is_key_down(&self) -> LC3Result<bool> {
        self.keydown_values
            .borrow_mut()
            .pop()
            .ok_or(LC3Error::Other(
                "Attempted to call getchar on empty key down vector".to_string(),
            ))
    }
}
