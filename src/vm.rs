use crate::command::Command;
use crate::condition_flags::{FL_NEG, FL_POS, FL_ZRO};
use crate::io::{IOHandle, RealIOHandle};
use crate::op::{handler, Op};
use crate::register::Register::{RCond, RCount, RPC, RR0, RR1, RR2, RR3, RR4, RR5, RR6, RR7};
use crate::register::{Register, NUM_REGISTERS};
use crate::trap::TrapCode;
use crate::utils::sign_extend;
use crate::wrapping_add;

const MEMORY_SIZE: usize = (u16::MAX as usize) + 1;
const PC_START: u16 = 0x3000;

pub struct VM<IOType>
where
    IOType: IOHandle,
{
    memory: [u16; MEMORY_SIZE],
    registers: [u16; NUM_REGISTERS],
    running: bool,
    io_handle: IOType,
}

impl VM<RealIOHandle> {
    // Want the default constructor to use a standard IO Handle, hence
    // the specific treatment.
    pub fn new() -> Self {
        Self::new_with_io(RealIOHandle {})
    }
}

impl<IOType> VM<IOType>
where
    IOType: IOHandle,
{
    // If there end up being more options to tweak might want to break out
    // a builder for this one, but right now this is fine.
    fn new_with_io(io_handle: IOType) -> Self {
        let memory = [0u16; MEMORY_SIZE];
        let registers = [0u16; NUM_REGISTERS];
        VM {
            memory,
            registers,
            running: false,
            io_handle,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        let pc_reg = RPC.index();
        self.registers[pc_reg] = PC_START;

        while self.running {
            let program_count = self.registers[pc_reg];
            self.registers[pc_reg] += 1;

            let command = Command::new(self.mem_read(program_count));
            self.run_command(&command)
        }
    }

    pub(crate) fn mem_read(&self, pos: u16) -> u16 {
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

    pub(crate) fn set_running(&mut self, val: bool) {
        self.running = val;
    }

    pub(crate) fn update_flags(&mut self, register_index: usize) {
        if register_index > NUM_REGISTERS - 1 {
            panic!("Register index {} too large", register_index);
        }

        let mut cond_flag = FL_POS;
        let value = self.registers[register_index];
        if value == 0 {
            cond_flag = FL_ZRO;
        } else if (value >> 15) == 1 {
            cond_flag = FL_NEG;
        };

        self.registers[RCond.index()] = cond_flag;
    }

    fn run_command(&mut self, command: &Command) {
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
    fn into_io_handle(self) -> IOType {
        self.io_handle
    }
}

#[cfg(test)]
mod test {
    use super::VM;
    use crate::command::Command;
    use crate::condition_flags::{FL_NEG, FL_POS, FL_ZRO};
    use crate::io::TestIOHandle;
    use crate::register::Register;
    const INITIAL_PC: u16 = 0x0F00;

    #[test]
    fn can_add() {
        let reg_cond = Register::RCond.index();
        let mut vm = VM::new();
        vm.registers[0] = 0;
        vm.registers[1] = 1;
        vm.registers[2] = 2;

        let command_results = vec![
            // Command to add register 1 to 2 and put result in register 0
            (0b0001_0000_0100_0010, 3, FL_POS),
            // Command to add -2 to register 1 and put result in register 0
            (0b0001_0000_0111_1110, 0xFFFF, FL_NEG),
            // Command to add -1 to register 1 and put result in register 0
            (0b0001_0000_0111_1111, 0, FL_ZRO),
        ];

        for (command, result, cond) in command_results {
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.registers[0], result);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_branch() {
        let reg_cond = Register::RCond.index();
        let reg_pc = Register::RPC.index();

        let command_cond_jump: Vec<(u16, u16, i16)> = vec![
            (0b0000_1000_0000_0100, FL_NEG, 4),
            (0b0000_1000_0000_0100, FL_POS, 0),
            (0b0000_1000_0000_0100, FL_ZRO, 0),
            (0b0000_0100_0000_1000, FL_NEG, 0),
            (0b0000_0100_0000_1000, FL_POS, 0),
            (0b0000_0100_0000_1000, FL_ZRO, 8),
            (0b0000_0010_0000_0010, FL_NEG, 0),
            (0b0000_0010_0000_0010, FL_POS, 2),
            (0b0000_0010_0000_0010, FL_ZRO, 0),
            (0b0000_0011_1111_1111, FL_POS, -1),
        ];

        for (raw_command, cond, jump) in command_cond_jump {
            let mut vm = VM::new();
            let command = Command::new(raw_command);
            vm.registers[reg_cond] = cond;
            vm.registers[reg_pc] = INITIAL_PC;
            vm.run_command(&command);
            assert_eq!(vm.registers[reg_pc] as i16, INITIAL_PC as i16 + jump);
        }
    }

    #[test]
    fn can_load() {
        let reg_pc = Register::RPC.index();
        let reg_cond = Register::RCond.index();

        let command_reg_val_offset_cond: Vec<(u16, usize, u16, i16, u16)> = vec![
            (0b0010_0000_0000_0000, 0, 0x0EFE, 0, FL_POS),
            (0b0010_0011_1111_1111, 1, 0xFEFE, -1, FL_NEG),
            (0b0010_0111_1111_1110, 3, 0x0000, -2, FL_ZRO),
        ];

        for (command, reg, val, offset, cond) in command_reg_val_offset_cond {
            let mut vm = VM::new();
            let command = Command::new(command);
            vm.registers[reg_pc] = INITIAL_PC;
            vm.mem_write((INITIAL_PC as i16 + offset) as u16, val);
            vm.run_command(&command);
            assert_eq!(vm.registers[reg], val);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_store() {
        let reg_pc = Register::RPC.index();
        let test_val = 0xFEFA;

        let command_reg_offset: Vec<(u16, usize, i16)> = vec![
            (0b0011_0000_0000_0000, 0, 0),
            (0b0011_0011_1111_1111, 1, -1),
            (0b0011_0111_1111_1110, 3, -2),
        ];

        for (command, reg, offset) in command_reg_offset {
            let mut vm = VM::new();
            let command = Command::new(command);
            vm.registers[reg] = test_val;
            vm.registers[reg_pc] = INITIAL_PC;
            vm.run_command(&command);
            assert_eq!(vm.mem_read((INITIAL_PC as i16 + offset) as u16), test_val);
        }
    }

    #[test]
    fn can_jump_register() {
        let reg_pc = Register::RPC.index();

        // Tests using base register mode
        // Tuple format: (command, register containing jump address, jump address)
        let base_test_cases: Vec<(u16, usize, u16)> = vec![
            (0b0100_0000_0100_0000, 1, 5),
            (0b0100_0000_1000_0000, 2, 13),
            (0b0100_0000_1100_0000, 3, 17),
        ];

        for (command, register, address) in base_test_cases {
            let mut vm = VM::new();
            let command = Command::new(command);
            vm.registers[reg_pc] = INITIAL_PC;
            vm.registers[register] = address;
            vm.run_command(&command);
            assert_eq!(vm.registers[Register::RR7.index()], INITIAL_PC);
            assert_eq!(vm.registers[reg_pc], address);
        }

        // Tests using offset mode
        // Tuple format: (command, offset)
        let offset_test_cases: Vec<(u16, i16)> = vec![
            (0b0100_1000_0000_0111, 7),
            (0b0100_1000_0000_0000, 0),
            (0b0100_1111_1111_1111, -1),
        ];

        for (command, offset) in offset_test_cases {
            let mut vm = VM::new();
            let command = Command::new(command);
            vm.registers[reg_pc] = INITIAL_PC;
            vm.run_command(&command);
            assert_eq!(vm.registers[Register::RR7.index()], INITIAL_PC);
            assert_eq!(vm.registers[reg_pc], ((INITIAL_PC as i16) + offset) as u16);
        }
    }

    #[test]
    fn can_and() {
        let mut vm = VM::new();
        let reg_cond = Register::RCond.index();
        vm.registers[0] = 0;
        vm.registers[1] = 1;
        vm.registers[2] = 0xFFFF;

        let command_result_cond = vec![
            // And register 1 with 2 and put result in register 0
            (0b0101_0000_0100_0010, 1, FL_POS),
            // And 2 with register 1 and put result in register 0
            (0b0101_0000_0110_0010, 0, FL_ZRO),
            // And -1 with register 2 and put result in register 0
            (0b0101_0000_1011_1111, 0xFFFF, FL_NEG),
        ];

        for (command, result, cond) in command_result_cond {
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.registers[0], result);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_load_register() {
        let reg_cond = Register::RCond.index();
        let base_reg = Register::RR4.index();
        let base_reg_val = 0x0FAE;

        let test_cases: Vec<(u16, i16, u16, u16)> = vec![
            (0b0110_0001_0011_1111, -1, 0xFFAF, FL_NEG),
            (0b0110_0001_0000_0000, 0, 0x0000, FL_ZRO),
            (0b0110_0001_0000_0001, 1, 0x000A, FL_POS),
        ];

        for (command, offset, mem_val, cond) in test_cases {
            let mut vm = VM::new();
            vm.registers[base_reg] = base_reg_val;
            vm.mem_write((base_reg_val as i16 + offset) as u16, mem_val);
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.registers[0], mem_val);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_store_register() {
        let base_reg = Register::RR4.index();
        let base_reg_val = 0x0FAE;
        let store_val = 0xFFAF;

        // Tuple format: (Command, offset value)
        let test_cases: Vec<(u16, i16)> = vec![
            (0b0111_0001_0011_1111, -1),
            (0b0111_0001_0000_0000, 0),
            (0b0111_0001_0000_0001, 1),
        ];

        for (command, offset) in test_cases {
            let mut vm = VM::new();
            vm.registers[base_reg] = base_reg_val;
            vm.registers[0] = store_val;
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(
                vm.mem_read((base_reg_val as i16 + offset) as u16),
                store_val
            );
        }
    }

    #[test]
    fn can_not() {
        let reg_cond = Register::RCond.index();
        let source_reg = Register::RR4.index();
        let target_reg = Register::RR3.index();

        // Tuple format: (Command, input, output, cond)
        let test_cases: Vec<(u16, u16, u16, u16)> = vec![
            (0b1001_0111_0011_1111, 0xFFFF, 0x0000, FL_ZRO),
            (0b1001_0111_0011_1111, 0x0000, 0xFFFF, FL_NEG),
            (0b1001_0111_0011_1111, 0xF000, 0x0FFF, FL_POS),
        ];

        for (command, input, output, cond) in test_cases {
            let mut vm = VM::new();
            vm.registers[source_reg] = input;
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.registers[target_reg], output);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_load_indirect() {
        let reg_cond = Register::RCond.index();
        let reg_pc = Register::RPC.index();
        let target_reg = Register::RR5.index();

        // Tuple format: (Command, pc offset, final address, stored value, cond)
        let test_cases: Vec<(u16, i16, u16, u16, u16)> = vec![
            (0b1010_1010_0000_0001, 1, 0xFAFA, 0x0AAA, FL_POS),
            (0b1010_1011_1111_1111, -1, 0xFAFA, 0x0000, FL_ZRO),
            (0b1010_1010_0000_0000, 0, 0xFAFA, 0x8000, FL_NEG),
        ];

        for (command, offset, address, value, cond) in test_cases {
            let mut vm = VM::new();
            vm.registers[reg_pc] = INITIAL_PC;
            vm.mem_write((INITIAL_PC as i16 + offset) as u16, address);
            vm.mem_write(address, value);
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.registers[target_reg], value);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_store_indirect() {
        let reg_pc = Register::RPC.index();
        let source_reg = Register::RR5.index();
        let store_val = 0xAAFA;

        // Tuple format: (Command, pc offset, final address)
        let test_cases: Vec<(u16, i16, u16)> = vec![
            (0b1011_1010_0000_0001, 1, 0xFAFA),
            (0b1011_1011_1111_1111, -1, 0xFAFA),
            (0b1011_1010_0000_0000, 0, 0xFAFA),
        ];

        for (command, offset, address) in test_cases {
            let mut vm = VM::new();
            vm.registers[reg_pc] = INITIAL_PC;
            vm.mem_write((INITIAL_PC as i16 + offset) as u16, address);
            vm.registers[source_reg] = store_val;
            let command = Command::new(command);
            vm.run_command(&command);
            assert_eq!(vm.mem_read(address), store_val);
        }
    }

    #[test]
    fn can_jump() {
        let reg_pc = Register::RPC.index();
        let source_reg = Register::RR7.index();
        let stored_pc = 0xFDBC;

        // Obviously, if this test's stored pc is equal to the initial pc, it's
        // pointless, so check to make sure this never happens. More of a test
        // of the test suite than anything.
        assert_ne!(INITIAL_PC, stored_pc);

        let mut vm = VM::new();
        vm.registers[reg_pc] = INITIAL_PC;
        vm.registers[source_reg] = stored_pc;
        let command = Command::new(0b1100_0001_1100_0000);
        vm.run_command(&command);
        assert_eq!(vm.registers[reg_pc], stored_pc);
    }

    #[test]
    fn can_load_effective_address() {
        let reg_pc = Register::RPC.index();
        let reg_cond = Register::RCond.index();
        let target_reg = Register::RR6.index();

        // Using a set value of initial_pc here, since it makes it easier
        // to check the cond register is setting correctly.
        let initial_pc = 0xFFFF;

        let test_cases = vec![
            (0b1110_1100_0000_0001, 1, FL_ZRO),
            (0b1110_1101_1111_1111, -1, FL_NEG),
            (0b1110_1100_0000_0010, 2, FL_POS),
        ];

        for (command, offset, cond) in test_cases {
            let mut vm = VM::new();
            vm.registers[reg_pc] = initial_pc;
            let command = Command::new(command);
            vm.run_command(&command);
            let target_val = (initial_pc as i16 + offset) as u16;
            assert_eq!(vm.registers[target_reg], target_val);
            assert_eq!(vm.registers[reg_cond], cond);
        }
    }

    #[test]
    fn can_trap_getchar() {
        let test_char = 'v';

        let mut io_handle = TestIOHandle::new();
        io_handle.add_key_press(test_char);
        let mut vm = VM::new_with_io(io_handle);

        let command = Command::new(0xF020);
        vm.run_command(&command);

        let reg_char = vm.registers[Register::RR0.index()] as u8 as char;
        assert_eq!(test_char, reg_char);
    }

    #[test]
    fn can_trap_out() {
        let test_char = 'w';
        let io_reg = Register::RR0.index();

        let io_handle = TestIOHandle::new();
        let mut vm = VM::new_with_io(io_handle);
        vm.registers[io_reg] = test_char as u16;

        let command = Command::new(0xF021);
        vm.run_command(&command);

        let mut outputs = vm.into_io_handle().get_test_outputs();
        assert!(outputs.len() == 1);
        assert!(outputs.pop() == Some(test_char));
    }

    #[test]
    fn can_trap_put_string() {
        let test_chars = vec!['a', 'b', 'c', 'd', 'e'];
        let io_reg_index = Register::RR0.index();
        let start_address = 0xCFFF;

        let io_handle = TestIOHandle::new();
        let mut vm = VM::new_with_io(io_handle);

        vm.registers[io_reg_index] = start_address;
        for (offset, test_char) in test_chars.iter().enumerate() {
            vm.memory[start_address as usize + offset] = *test_char as u16;
        }

        let command = Command::new(0xF022);
        vm.run_command(&command);

        let mut outputs = vm.into_io_handle().get_test_outputs();
        assert_eq!(outputs, test_chars);
    }

    #[test]
    fn can_trap_in() {
        let test_char = 'w';

        let mut io_handle = TestIOHandle::new();
        io_handle.add_key_press(test_char);
        let mut vm = VM::new_with_io(io_handle);

        let command = Command::new(0xF023);
        vm.run_command(&command);

        let mut outputs = vm.into_io_handle().get_test_outputs();
        assert!(outputs.len() == 1);
        assert!(outputs.pop() == Some(test_char));
    }

    #[test]
    fn can_trap_put_byte_string() {
        let test_chars = vec!['a', 'b', 'c', 'd', 'e'];
        let io_reg_index = Register::RR0.index();
        let start_address = 0xCFFF;

        let io_handle = TestIOHandle::new();
        let mut vm = VM::new_with_io(io_handle);
        vm.registers[io_reg_index] = start_address;

        for (pos, test_char) in test_chars.iter().enumerate() {
            let mut mask = *test_char as u16;
            let is_right = (pos % 2) == 1;
            if is_right {
                mask = mask << 8;
            };

            let mem_offset = pos / 2;
            vm.memory[start_address as usize + mem_offset] |= mask;
        }

        let command = Command::new(0xF024);
        vm.run_command(&command);

        let outputs = vm.into_io_handle().get_test_outputs();
        assert_eq!(outputs, test_chars);
    }

    #[test]
    fn can_trap_halt() {
        let mut vm = VM::new();
        vm.running = true;
        let command = Command::new(0xF025);
        vm.run_command(&command);
        println!("{:?}", vm.running);
    }

    #[test]
    fn can_update_flags() {
        let mut vm = VM::new();
        let value_flag_pairs = vec![(0u16, FL_ZRO), (0x0001, FL_POS), (0x8111, FL_NEG)];

        let test_register: usize = 0;
        let flags_register: usize = Register::RCond.index();
        for (value, flag) in value_flag_pairs {
            vm.registers[test_register] = value;
            vm.update_flags(test_register);
            assert_eq!(vm.registers[flags_register], flag);
        }
    }
}
