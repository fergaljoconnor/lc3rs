use crate::command::Command;
use crate::condition_flags::{FL_NEG, FL_POS, FL_ZRO};
use crate::io::TestIOHandle;
use crate::register::Register;
use crate::register::Register::{RCond, RPC};
use crate::vm::VM;

const INITIAL_PC: u16 = 0x0F00;

#[test]
fn can_add() {
    let mut vm = VM::new();
    vm.reg_index_write(0, 0);
    vm.reg_index_write(1, 1);
    vm.reg_index_write(2, 2);

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
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_index_read(0), result);
        assert_eq!(vm.reg_read(Register::RCond), cond);
    }
}

#[test]
fn can_branch() {
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
        vm.reg_write(Register::RCond, cond);
        vm.reg_write(Register::RPC, INITIAL_PC);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_read(Register::RPC) as i16, INITIAL_PC as i16 + jump);
    }
}

#[test]
fn can_load() {
    let command_reg_val_offset_cond: Vec<(u16, usize, u16, i16, u16)> = vec![
        (0b0010_0000_0000_0000, 0, 0x0EFE, 0, FL_POS),
        (0b0010_0011_1111_1111, 1, 0xFEFE, -1, FL_NEG),
        (0b0010_0111_1111_1110, 3, 0x0000, -2, FL_ZRO),
    ];

    for (command, reg, val, offset, cond) in command_reg_val_offset_cond {
        let mut vm = VM::new();
        let command = Command::new(command);
        vm.reg_write(RPC, INITIAL_PC);
        vm.mem_write((INITIAL_PC as i16 + offset) as u16, val);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_index_read(reg as u8), val);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_store() {
    let test_val = 0xFEFA;

    let command_reg_offset: Vec<(u16, u8, i16)> = vec![
        (0b0011_0000_0000_0000, 0, 0),
        (0b0011_0011_1111_1111, 1, -1),
        (0b0011_0111_1111_1110, 3, -2),
    ];

    for (command, reg, offset) in command_reg_offset {
        let mut vm = VM::new();
        let command = Command::new(command);
        vm.reg_index_write(reg, test_val);
        vm.reg_write(RPC, INITIAL_PC);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.mem_read((INITIAL_PC as i16 + offset) as u16), test_val);
    }
}

#[test]
fn can_jump_register() {
    // Tests using base register mode
    // Tuple format: (command, register containing jump address, jump address)
    let base_test_cases: Vec<(u16, u8, u16)> = vec![
        (0b0100_0000_0100_0000, 1, 5),
        (0b0100_0000_1000_0000, 2, 13),
        (0b0100_0000_1100_0000, 3, 17),
    ];

    for (command, register, address) in base_test_cases {
        let mut vm = VM::new();
        let command = Command::new(command);
        vm.reg_write(RPC, INITIAL_PC);
        vm.reg_index_write(register, address);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_read(Register::RR7), INITIAL_PC);
        assert_eq!(vm.reg_read(RPC), address);
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
        vm.reg_write(RPC, INITIAL_PC);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_read(Register::RR7), INITIAL_PC);
        assert_eq!(vm.reg_read(RPC), ((INITIAL_PC as i16) + offset) as u16);
    }
}

#[test]
fn can_and() {
    let mut vm = VM::new();
    vm.reg_index_write(0, 0);
    vm.reg_index_write(1, 1);
    vm.reg_index_write(2, 0xFFFF);

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
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_index_read(0), result);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_load_register() {
    let base_reg = Register::RR4;
    let base_reg_val = 0x0FAE;

    let test_cases: Vec<(u16, i16, u16, u16)> = vec![
        (0b0110_0001_0011_1111, -1, 0xFFAF, FL_NEG),
        (0b0110_0001_0000_0000, 0, 0x0000, FL_ZRO),
        (0b0110_0001_0000_0001, 1, 0x000A, FL_POS),
    ];

    for (command, offset, mem_val, cond) in test_cases {
        let mut vm = VM::new();
        vm.reg_write(base_reg, base_reg_val);
        vm.mem_write((base_reg_val as i16 + offset) as u16, mem_val);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_index_read(0), mem_val);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_store_register() {
    let base_reg = Register::RR4;
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
        vm.reg_write(base_reg, base_reg_val);
        vm.reg_index_write(0, store_val);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        assert_eq!(
            vm.mem_read((base_reg_val as i16 + offset) as u16),
            store_val
        );
    }
}

#[test]
fn can_not() {
    let source_reg = Register::RR4;
    let target_reg = Register::RR3;

    // Tuple format: (Command, input, output, cond)
    let test_cases: Vec<(u16, u16, u16, u16)> = vec![
        (0b1001_0111_0011_1111, 0xFFFF, 0x0000, FL_ZRO),
        (0b1001_0111_0011_1111, 0x0000, 0xFFFF, FL_NEG),
        (0b1001_0111_0011_1111, 0xF000, 0x0FFF, FL_POS),
    ];

    for (command, input, output, cond) in test_cases {
        let mut vm = VM::new();
        vm.reg_write(source_reg, input);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_read(target_reg), output);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_load_indirect() {
    let target_reg = Register::RR5;

    // Tuple format: (Command, pc offset, final address, stored value, cond)
    let test_cases: Vec<(u16, i16, u16, u16, u16)> = vec![
        (0b1010_1010_0000_0001, 1, 0xFAFA, 0x0AAA, FL_POS),
        (0b1010_1011_1111_1111, -1, 0xFAFA, 0x0000, FL_ZRO),
        (0b1010_1010_0000_0000, 0, 0xFAFA, 0x8000, FL_NEG),
    ];

    for (command, offset, address, value, cond) in test_cases {
        let mut vm = VM::new();
        vm.reg_write(RPC, INITIAL_PC);
        vm.mem_write((INITIAL_PC as i16 + offset) as u16, address);
        vm.mem_write(address, value);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.reg_read(target_reg), value);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_store_indirect() {
    let source_reg = Register::RR5;
    let store_val = 0xAAFA;

    // Tuple format: (Command, pc offset, final address)
    let test_cases: Vec<(u16, i16, u16)> = vec![
        (0b1011_1010_0000_0001, 1, 0xFAFA),
        (0b1011_1011_1111_1111, -1, 0xFAFA),
        (0b1011_1010_0000_0000, 0, 0xFAFA),
    ];

    for (command, offset, address) in test_cases {
        let mut vm = VM::new();
        vm.reg_write(RPC, INITIAL_PC);
        vm.mem_write((INITIAL_PC as i16 + offset) as u16, address);
        vm.reg_write(source_reg, store_val);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        assert_eq!(vm.mem_read(address), store_val);
    }
}

#[test]
fn can_jump() {
    let source_reg = Register::RR7;
    let stored_pc = 0xFDBC;

    // Obviously, if this test's stored pc is equal to the initial pc, it's
    // pointless, so check to make sure this never happens. More of a test
    // of the test suite than anything.
    assert_ne!(INITIAL_PC, stored_pc);

    let mut vm = VM::new();
    vm.reg_write(RPC, INITIAL_PC);
    vm.reg_write(source_reg, stored_pc);
    let command = Command::new(0b1100_0001_1100_0000);
    vm.run_command(&command).unwrap();
    assert_eq!(vm.reg_read(RPC), stored_pc);
}

#[test]
fn can_load_effective_address() {
    let target_reg = Register::RR6;

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
        vm.reg_write(RPC, initial_pc);
        let command = Command::new(command);
        vm.run_command(&command).unwrap();
        let target_val = (initial_pc as i16 + offset) as u16;
        assert_eq!(vm.reg_read(target_reg), target_val);
        assert_eq!(vm.reg_read(RCond), cond);
    }
}

#[test]
fn can_trap_getchar() {
    let test_char = 'v';

    let mut io_handle = TestIOHandle::new();
    io_handle.add_key_press(test_char);
    let mut vm = VM::new_with_io(io_handle);

    let command = Command::new(0xF020);
    vm.run_command(&command).unwrap();

    let reg_char = vm.reg_read(Register::RR0) as u8 as char;
    assert_eq!(test_char, reg_char);
}

#[test]
fn can_trap_out() {
    let test_char = 'w';
    let io_reg = Register::RR0;

    let io_handle = TestIOHandle::new();
    let mut vm = VM::new_with_io(io_handle);
    vm.reg_write(io_reg, test_char as u16);

    let command = Command::new(0xF021);
    vm.run_command(&command).unwrap();

    let mut outputs = vm.into_io_handle().get_test_outputs();
    assert!(outputs.len() == 1);
    assert!(outputs.pop() == Some(test_char));
}

#[test]
fn can_trap_put_string() {
    let test_chars = vec!['a', 'b', 'c', 'd', 'e'];
    let io_reg = Register::RR0;
    let start_address = 0xCFFF;

    let io_handle = TestIOHandle::new();
    let mut vm = VM::new_with_io(io_handle);

    vm.reg_write(io_reg, start_address);
    for (offset, test_char) in test_chars.iter().enumerate() {
        vm.mem_write(start_address + offset as u16, *test_char as u16);
    }

    let command = Command::new(0xF022);
    vm.run_command(&command).unwrap();

    let outputs = vm.into_io_handle().get_test_outputs();
    assert_eq!(outputs, test_chars);
}

#[test]
fn can_trap_in() {
    let test_char = 'w';

    let mut io_handle = TestIOHandle::new();
    io_handle.add_key_press(test_char);
    let mut vm = VM::new_with_io(io_handle);

    let command = Command::new(0xF023);
    vm.run_command(&command).unwrap();

    let mut outputs = vm.into_io_handle().get_test_outputs();
    assert!(outputs.len() == 1);
    assert!(outputs.pop() == Some(test_char));
}

#[test]
fn can_trap_put_byte_string() {
    let test_chars = vec!['a', 'b', 'c', 'd', 'e'];
    let io_reg = Register::RR0;
    let start_address = 0xCFFF;

    let io_handle = TestIOHandle::new();
    let mut vm = VM::new_with_io(io_handle);
    vm.reg_write(io_reg, start_address);

    for (pos, test_char) in test_chars.iter().enumerate() {
        let mut mask = *test_char as u16;
        let is_right = (pos % 2) == 1;
        if is_right {
            mask = mask << 8;
        };

        let mem_offset = pos / 2;
        let address = start_address + mem_offset as u16;
        let new_val =  vm.mem_read(address) | mask;
        vm.mem_write(address, new_val);
    }

    let command = Command::new(0xF024);
    vm.run_command(&command).unwrap();

    let outputs = vm.into_io_handle().get_test_outputs();
    assert_eq!(outputs, test_chars);
}

#[test]
fn can_trap_halt() {
    let mut vm = VM::new();
    vm.set_running(true);
    let command = Command::new(0xF025);
    vm.run_command(&command).unwrap();
    assert_eq!(vm.get_running(), false);
}

#[test]
fn can_update_flags() {
    let mut vm = VM::new();
    let value_flag_pairs = vec![(0u16, FL_ZRO), (0x0001, FL_POS), (0x8111, FL_NEG)];

    let test_reg = Register::RR0;
    let flags_reg = Register::RCond;
    for (value, flag) in value_flag_pairs {
        vm.reg_write(test_reg, value);
        vm.update_flags(test_reg.index());
        assert_eq!(vm.reg_read(flags_reg), flag);
    }
}
