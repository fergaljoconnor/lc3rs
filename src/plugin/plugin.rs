use crate::vm::VM;
use crate::io::IOHandle;

#[derive(Debug)]
pub enum Event {
    Command {bytes: u16},
    MemRead{location: u16, value: u16},
    MemWrite{location: u16, value: u16},
    RegRead{index: u8, value: u16},
    RegWrite{index: u8, value: u16},
}

pub trait Plugin
{
    fn handle_event<IOType: IOHandle>(&mut self, vm: &mut VM<IOType>, event: &Event);
}
