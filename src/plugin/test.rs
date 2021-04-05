use std::cell::RefCell;
use std::rc::Rc;

use crate::error::LC3Result;
use crate::io::IOHandle;
use crate::vm::VM;
use super::Event::*;
use super::{Event, Plugin};

type EventsReference = Rc<RefCell<Vec<Event>>>;

struct TestPlugin {
    events: EventsReference,
}

impl TestPlugin {
    fn new() -> Self {
        Self {
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn get_events_ref(&self) -> EventsReference {
        self.events.clone()
    }
}

impl<IOType: IOHandle> Plugin<IOType> for TestPlugin {
    fn handle_event(&mut self, _vm: &mut VM<IOType>, event: &Event) -> LC3Result<()> {
        self.events.borrow_mut().push(event.clone());
        Ok(())
    }
}

#[test]
fn can_push_events_to_plugin() -> LC3Result<()> {
    let test_events = vec![
        Command { bytes: 1 },
        CharGet { ch: 'a' },
        CharPut { ch: 'b' },
        KeyDownGet { value: true },
        MemGet {
            location: 2,
            value: 3,
        },
        MemSet {
            location: 4,
            value: 4,
        },
        RegGet { index: 5, value: 6 },
        RegSet { index: 7, value: 8 },
        RunningGet { value: false },
        RunningSet { value: true },
    ];

    let mut plugin = TestPlugin::new();
    let mut vm = VM::new();
    let events_ref = plugin.get_events_ref();

    for event in &test_events {
        plugin.handle_event(&mut vm, event)?;
    }

    let written_events = events_ref.borrow().clone();
    assert_eq!(test_events, written_events);

    Ok(())
}
