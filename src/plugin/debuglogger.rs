use std::io::Write;

use crate::command::Command;
use crate::op::Op;
use crate::io::IOHandle;
use crate::vm::VM;
use super::{Event, Plugin};

pub struct DebugLogger<Sink: Write> {
    sink: Sink,
}

impl<Sink: Write> DebugLogger<Sink> {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }

    #[cfg(test)]
    fn into_sink(self) -> Sink {
        self.sink
    }
}

impl<Sink: Write, IOType: IOHandle> Plugin<IOType> for DebugLogger<Sink> {
    fn handle_event(&mut self, _: &mut VM<IOType>, event: &Event) {
        // TODO: Proper error propagation
        self.sink
            .write(debug_format(event).as_bytes())
            .expect("Debug Logger Encountered an issue writing to log");
        self.sink.write(b"\n").expect("Debug Logger Encountered an issue writing to log");
        self.sink
            .flush()
            .expect("Debug Logger Encountered an issue flushing log");
    }
}

fn debug_format(event: &Event) -> String {
    match event {
        Event::Command{bytes} => debug_format_command(*bytes),
        _ => format!("{:?}", event)
    }
}

fn debug_format_command(bytes: u16) -> String {
    let command = Command::new(bytes);
    let op = Op::from_int(command.op_code().unwrap());
    format!("Command: {{ bytes: {:16b}, op: {:?} }}", bytes, op)
}


#[cfg(test)]
mod test {
    use super::super::{Event, Plugin};
    use super::DebugLogger;
    use crate::vm::VM;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    #[test]
    fn can_handle_event() {
        let test_cases = vec![
            Event::MemGet {
                location: 1,
                value: 2,
            },
            Event::MemSet {
                location: 3,
                value: 4,
            },
            Event::RegGet { index: 5, value: 6 },
            Event::RegSet { index: 7, value: 8 },
        ];

        for event in test_cases {
            let sink = Cursor::new(Vec::<u8>::new());
            let mut logger = DebugLogger::new(sink);
            let mut vm = VM::new();

            logger.handle_event(&mut vm, &event);

            let mut sink = logger.into_sink();
            sink.seek(SeekFrom::Start(0)).unwrap();
            let mut output = String::new();
            sink.read_to_string(&mut output).unwrap();

            let expected = format!("{:?}\n", event);
            assert_eq!(output, expected);
        }
    }
}
