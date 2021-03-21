use crate::command::Command;
use crate::condition_flags::{FL_NEG, FL_POS, FL_ZRO};
use crate::io::{IOHandle, RealIOHandle};
use crate::op::{handler, Op};
use crate::plugin::{Event, Plugin};
use crate::register::Register::{RCond, RCount, RPC, RR0, RR1, RR2, RR3, RR4, RR5, RR6, RR7};
use crate::register::{Register, NUM_REGISTERS};
use crate::trap::TrapCode;
use crate::utils::sign_extend;
use crate::wrapping_add;

const MEMORY_SIZE: usize = (u16::MAX as usize) + 1;

const PC_START: u16 = 0x3000; // Initial program counter

// Mem Mapped Register Locations
// There are 3 registers listed in the spec
// (https://courses.engr.illinois.edu/ece411/fa2019/mp/LC3b_ISA.pdf
// or https://justinmeiners.github.io/lc3-vm/supplies/lc3-isa.pdf) we don't
// implement here yet, the display status register, display data register and
// the machine control register.
const KB_STATUS_POS: u16 = 0xFE00; // Keyboard Status Register
const KB_DATA_POS: u16 = 0xFE02; // Keyboard Data Register

pub struct VM<IOType: IOHandle>
{
    // TODO: Splitting the state between a VM state component and
    // a  plugin manager component would make it easier for the compiler to
    // reason about mutability during plugin notifications and push some of
    // the fiddly plugin management logic into a class where it's more relevant.
    memory: [u16; MEMORY_SIZE],
    registers: [u16; NUM_REGISTERS],
    running: bool,
    io_handle: IOType,
    plugins: Option<Vec<Box<dyn Plugin<IOType>>>>,
    notifying_plugins: bool,
}

impl VM<RealIOHandle> {
    // Want the default constructor to use a standard IO Handle, hence
    // the specific treatment.
    pub fn new() -> Self {
        Self::new_with_io(RealIOHandle {})
    }
}

impl<IOType: IOHandle> VM<IOType>
{
    // If there end up being more options to tweak might want to break out
    // a builder for this one, but right now this is fine.
    pub(crate) fn new_with_io(io_handle: IOType) -> Self {
        let memory = [0u16; MEMORY_SIZE];
        let registers = [0u16; NUM_REGISTERS];
        VM {
            memory,
            registers,
            running: false,
            io_handle,
            plugins: Some(Vec::new()),
            notifying_plugins: false,
        }
    }

    pub fn run(&mut self) {
        self.set_running(true);
        self.reg_write(RPC, PC_START);

        while self.get_running() {
            let program_count = self.reg_read(RPC);
            self.reg_write(RPC, program_count + 1);

            let command = Command::new(self.mem_read(program_count));
            self.run_command(&command)
        }
    }

    pub fn load_program(&mut self, program: &Vec<u16>) {
        let max_len = MEMORY_SIZE - PC_START as usize;
        if program.len() > max_len {
            panic!(
                "Program length {} exceeds maximum allowed size {}",
                program.len(),
                max_len
            )
        }

        for (index, instruction) in program.iter().enumerate() {
            self.mem_write(PC_START + index as u16, *instruction);
        }
    }

    pub(crate) fn mem_read(&mut self, pos: u16) -> u16 {
        // Deal with the mem-mapped device registers
        if pos == KB_STATUS_POS {
            if self.io_handle.is_key_down() {
                // TODO: Right now, I think there's a bug here. If the key
                // being pressed is not a key handled by getchar()
                // then the vm will fill the status register and pause
                // waiting for the user to press one of those keys before
                // actually doing anything. Not a show stopper, but one to
                // watch.
                self.mem_write(KB_STATUS_POS, 1 << 15);
                self.mem_write(KB_DATA_POS, self.io_handle.getchar() as u16);
            } else {
                self.mem_write(KB_STATUS_POS, 0);
            }
        };
        self.memory[pos as usize]
    }

    pub(crate) fn mem_write(&mut self, pos: u16, val: u16) {
        self.memory[pos as usize] = val
    }

    pub(crate) fn reg_read(&self, reg: Register) -> u16 {
        self.reg_index_read(reg.to_u8())
    }

    pub(crate) fn reg_write(&mut self, reg: Register, val: u16) {
        self.reg_index_write(reg.to_u8(), val);
    }

    pub(crate) fn reg_index_read(&self, index: u8) -> u16 {
        self.registers[index as usize]
    }

    pub(crate) fn reg_index_write(&mut self, index: u8, val: u16) {
        self.registers[index as usize] = val;
    }

    pub(crate) fn putchar(&self, ch: char) {
        self.io_handle.putchar(ch)
    }

    pub(crate) fn getchar(&self) -> char {
        self.io_handle.getchar()
    }

    pub(crate) fn get_running(&mut self) -> bool {
        self.running
    }

    pub(crate) fn set_running(&mut self, val: bool) {
        self.running = val;
    }

    pub(crate) fn update_flags(&mut self, register_index: usize) {
        if register_index > NUM_REGISTERS - 1 {
            panic!("Register index {} too large", register_index);
        }

        let mut cond_flag = FL_POS;
        let value = self.reg_index_read(register_index as u8);
        if value == 0 {
            cond_flag = FL_ZRO;
        } else if (value >> 15) == 1 {
            cond_flag = FL_NEG;
        };

        self.reg_write(RCond, cond_flag);
    }

    pub(crate) fn notify_plugins(&mut self, event: &Event) {
        // This memory swapping dance prevents a safety issue.
        // Basically, if we were iterating over the plugins vector contained
        // in the VM while also allowing the plugins to mutate the VM while
        // they were handling the event, then the plugins could theoretically
        // mutate their own vector while it is being iterated over, which is
        // obviously bad for business.
        //
        // The other issue here is loops. Imagine you have two
        // plugins, one has the job of always setting register 0 to 1 (plugin 1)
        // and the other has the job of setting it to 2 (plugin 2). These
        // plugins are set up so whenever they receive a reg_write event to
        // register 0, they overwrite it with their value. So if these
        // events can be generated in the middle of the notifications
        // loop plugin 1 setting the value will trigger another iteration
        // of the loop. Even if plugin 1 somehow didn't cause a loop by putting
        // reg_read/ reg_write notifications out there, the interaction
        // of plugin 1 and plugin 2 fighting over the value will. If you
        // prevent new events being generated while the notification loop is
        // running, it prevents the issue, at the cost of not being able to
        // get notifications on what the other plugins are doing.

        if self.plugins.is_none() {
            // We're in the notifications loop, don't push the event
            return;
        }

        let mut plugins_option = None;
        std::mem::swap(&mut plugins_option, &mut self.plugins);
        let mut plugins = plugins_option.unwrap();

        for plugin in &mut plugins  {
            plugin.handle_event(self, event)
        }

        plugins_option = Some(plugins);
        std::mem::swap(&mut plugins_option, &mut self.plugins);
    }

    pub(crate) fn run_command(&mut self, command: &Command) {
        let op = Op::from_int(command.op_code());
        match op {
            Op::Br => handler::branch(self, command),
            Op::Add => handler::add(self, command),
            Op::Ld => handler::load(self, command),
            Op::St => handler::store(self, command),
            Op::Jsr => handler::jump_register(self, command),
            Op::And => handler::and(self, command),
            Op::Ldr => handler::load_register(self, command),
            Op::Str => handler::store_register(self, command),
            Op::Rti => handler::rti(self, command),
            Op::Not => handler::not(self, command),
            Op::Ldi => handler::load_indirect(self, command),
            Op::Sti => handler::store_indirect(self, command),
            Op::Jmp => handler::jump(self, command),
            Op::Res => handler::reserved(self, command),
            Op::Lea => handler::load_effective_address(self, command),
            Op::Trap => handler::trap(self, command),
        }
    }

    #[cfg(test)]
    pub(crate) fn into_io_handle(self) -> IOType {
        self.io_handle
    }
}

#[cfg(test)]
mod test {
    use super::VM;
    use crate::condition_flags::{FL_NEG, FL_POS, FL_ZRO};
    use crate::io::TestIOHandle;
    use crate::register::Register::RCond;

    #[test]
    fn can_update_flags() {
        // Tuple format: (Register value, Expected Flag)
        let test_cases = vec![(0u16, FL_ZRO), (0x0001, FL_POS), (0x8111, FL_NEG)];

        let test_reg = 0;
        for (value, flag) in test_cases {
            let mut vm = VM::new();
            vm.reg_index_write(test_reg, value);
            vm.update_flags(test_reg as usize);
            assert_eq!(vm.reg_read(RCond), flag);
        }
    }

    #[test]
    fn can_read_memmapped_registers() {
        let test_char = 'q';

        let mut io_handle = TestIOHandle::new();
        io_handle.add_keydown_response(true);
        io_handle.add_key_press(test_char);
        let mut vm = VM::new_with_io(io_handle);

        // Note in case I'm changing this in the future. The ordering
        // here is important. The read of the status register and
        // positive response is what triggers the update of the data
        // register, so if the order of the statements is flipped, the data
        // register read fails (and should, since we're not on a physical
        // machine there's nothing independently updating the registers
        // on its own schedule).
        assert_eq!(vm.mem_read(super::KB_STATUS_POS), 1 << 15);
        assert_eq!(vm.mem_read(super::KB_DATA_POS) as u8 as char, test_char);
    }

    #[test]
    fn can_run_program() {
        let mut program: Vec<u16> = vec![
            // Write (incremented program counter + 2) into RR0
            0b1110_0000_0000_0010,
            // Print the string starting at the address in RR0
            0xF022,
            // Halt
            0xF025,
        ];

        let test_string = "Hello world!";
        let char_vals = test_string.chars().map(|ch| ch as u16);
        program.extend(char_vals);

        let io_handle = TestIOHandle::new();
        let mut vm = VM::new_with_io(io_handle);
        vm.load_program(&program);
        vm.run();

        let io_handle = vm.into_io_handle();
        let outputs: String = io_handle.get_test_outputs().iter().collect();
        assert_eq!(test_string.to_string(), outputs);
    }
}
