use device_query::{DeviceQuery, DeviceState};
use super::io::{getchar, putchar};
#[cfg(test)]
use std::cell::RefCell;

// TODO: Maybe the dependency should be flipped here, so this trait should
// be part of the VM module. It's the VM's needs that actually determine
// what this interface should do.
pub trait IOHandle {
    fn getchar(&self) -> char;
    fn putchar(&self, ch: char);
    fn is_key_down(&self) -> bool;
}

pub struct RealIOHandle { device_state: DeviceState }

impl RealIOHandle {
    pub fn new() -> Self {
        Self { device_state: DeviceState::new() }
    }
}

impl IOHandle for RealIOHandle {
    fn getchar(&self) -> char {
        getchar()
    }

    fn putchar(&self, ch: char) {
        putchar(ch)
    }

    fn is_key_down(&self) -> bool {
        self.device_state.get_keys().is_empty()
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

    pub(crate) fn add_key_presses(&mut self, chars: Vec<char>) {
        self.key_presses.borrow_mut().extend(chars);
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
    fn getchar(&self) -> char {
        self.key_presses
            .borrow_mut()
            .pop()
            .expect("getchar called on empty test key_presses vec")
    }

    fn putchar(&self, ch: char) {
        self.outputs.borrow_mut().push(ch);
    }

    fn is_key_down(&self) -> bool {
        self.keydown_values
            .borrow_mut()
            .pop()
            .expect("is_key_down called on empty keydown_values vec")
    }
}
