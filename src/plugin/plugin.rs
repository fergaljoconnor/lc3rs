use crate::error::LC3Result;
use crate::io::IOHandle;
use crate::vm::VM;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Command { bytes: u16 },
    CharGet { ch: char },
    CharPut { ch: char },
    KeyDownGet { value: bool },
    MemGet { location: u16, value: u16 },
    MemSet { location: u16, value: u16 },
    RegGet { index: u8, value: u16 },
    RegSet { index: u8, value: u16 },
    RunningGet { value: bool },
    RunningSet { value: bool },
}

pub trait Plugin<IOType: IOHandle> {
    fn handle_event(&mut self, vm: &mut VM<IOType>, event: &Event) -> LC3Result<()>;
}
