use std::io::Write;

use super::{Event, Plugin};
use crate::command::Command;
use crate::error::{BoxErrors, LC3Result};
use crate::io::IOHandle;
use crate::op::Op;
use crate::vm::VM;

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
    fn handle_event(&mut self, _: &mut VM<IOType>, event: &Event) -> LC3Result<()> {
        self.sink
            .write(debug_format(event)?.as_bytes())
            .map_plugin_error()?;
        self.sink.write(b"\n").map_plugin_error()?;
        self.sink.flush().map_plugin_error()?;

        Ok(())
    }
}

fn debug_format(event: &Event) -> LC3Result<String> {
    let formatted = match event {
        Event::Command { bytes } => debug_format_command(*bytes)?,
        _ => format!("{:?}", event),
    };

    Ok(formatted)
}

fn debug_format_command(bytes: u16) -> LC3Result<String> {
    let command = Command::new(bytes);
    let op = Op::from_int(command.op_code()?);
    Ok(format!("Command: {{ bytes: {:16b}, op: {:?} }}", bytes, op))
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use super::super::{Event, Plugin};
    use super::DebugLogger;

    use crate::error::{BoxErrors, LC3Result};
    use crate::vm::VM;

    #[test]
    fn can_handle_event() -> LC3Result<()> {
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

            logger.handle_event(&mut vm, &event)?;

            let mut sink = logger.into_sink();
            sink.seek(SeekFrom::Start(0)).map_io_error()?;
            let mut output = String::new();
            sink.read_to_string(&mut output).map_io_error()?;

            let expected = format!("{:?}\n", event);
            assert_eq!(output, expected);
        }

        Ok(())
    }
}
