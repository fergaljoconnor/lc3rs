#[cfg(test)]
use std::cell::RefCell;
use super::io::{getchar, putchar};

pub trait IOHandle {
    fn getchar(&self) -> char;
    fn putchar(&self, ch: char);
}

pub(crate) struct RealIOHandle;

impl IOHandle for RealIOHandle {
    fn getchar(&self) -> char {
        getchar()
    }

    fn putchar(&self, ch: char) {
        putchar(ch)
    }
}

#[cfg(test)]
pub(crate) struct TestIOHandle {
    key_presses: RefCell<Vec<char>>,
    outputs: RefCell<Vec<char>>,
}

#[cfg(test)]
impl TestIOHandle {
    pub(crate) fn new() -> Self {
        Self {
            key_presses: RefCell::new(Vec::new()),
            outputs: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn add_key_press(&mut self, ch: char) {
        self.key_presses.borrow_mut().push(ch);
    }

    pub(crate) fn add_key_presses(&mut self, chars: Vec<char>) {
        self.key_presses.borrow_mut().extend(chars);
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
}
