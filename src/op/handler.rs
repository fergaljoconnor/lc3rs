use crate::command::Command;
use crate::io::{IOHandle};
use crate::register::Register::{RCond, RPC, RR7};
use crate::trap::TrapCode;
use crate::utils::sign_extend;
use crate::vm::VM;
use crate::wrapping_add;
use super::trap_handler as handle_trap;

pub(crate) fn branch<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let test_flag = command.bit_slice(4, 6);
    let flags = vm.reg_read(RCond);
    let will_branch = (flags & test_flag) != 0;

    if will_branch {
        let offset = sign_extend(command.bit_slice(7, 15), 9);
        let new_pc = wrapping_add!(vm.reg_read(RPC), offset);
        vm.reg_write(RPC, new_pc);
    }
}

pub(crate) fn add<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target_reg = command.bit_slice(4, 6) as u8;
    let left = vm.reg_index_read(command.bit_slice(7, 9) as u8);
    let immediate = command.bit_slice(10, 10) == 1;

    let right = if immediate {
        sign_extend(command.bit_slice(11, 15), 5)
    } else {
        vm.reg_index_read(command.bit_slice(13, 15) as u8)
    };

    vm.reg_index_write(target_reg, wrapping_add!(left, right));
    vm.update_flags(target_reg as usize);
}

pub(crate) fn load<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target_reg = command.bit_slice(4, 6) as u8;
    let offset = sign_extend(command.bit_slice(7, 15), 9);
    let pc = vm.reg_read(RPC);
    let address = wrapping_add!(pc, offset);
    vm.reg_index_write(target_reg, vm.mem_read(address));
    vm.update_flags(target_reg.into());
}

pub(crate) fn store<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let source = command.bit_slice(4, 6) as u8;
    let offset = sign_extend(command.bit_slice(7, 15), 9);
    let target = wrapping_add!(vm.reg_read(RPC), offset);
    vm.mem_write(target, vm.reg_index_read(source));
}

pub(crate) fn jump_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    // Save program counter
    vm.reg_write(RR7, vm.reg_read(RPC));

    let offset_mode = command.bit_slice(4, 4) == 1;

    let destination = if offset_mode {
        let offset = sign_extend(command.bit_slice(5, 15), 11);
        wrapping_add!(vm.reg_read(RPC), offset)
    } else {
        let source_reg = command.bit_slice(7, 9) as u8;
        vm.reg_index_read(source_reg)
    };

    vm.reg_write(RPC, destination);
}

pub(crate) fn and<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target_reg = command.bit_slice(4, 6) as u8;
    let left = vm.reg_index_read(command.bit_slice(7, 9) as u8);
    let immediate = command.bit_slice(10, 10) == 1;

    let right = if immediate {
        sign_extend(command.bit_slice(11, 15), 5)
    } else {
        vm.reg_index_read(command.bit_slice(13, 15) as u8)
    };

    vm.reg_index_write(target_reg, left & right);
    vm.update_flags(target_reg.into());
}

pub(crate) fn load_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target = command.bit_slice(4, 6) as u8;
    let base = command.bit_slice(7, 9) as u8;
    let offset = sign_extend(command.bit_slice(10, 15), 6);
    let address = wrapping_add!(vm.reg_index_read(base), offset);
    vm.reg_index_write(target, vm.mem_read(address));
    vm.update_flags(target.into());
}

pub(crate) fn store_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let source = command.bit_slice(4, 6) as u8;
    let base_register = command.bit_slice(7, 9) as u8;
    let offset = sign_extend(command.bit_slice(10, 15), 6);
    let address = wrapping_add!(vm.reg_index_read(base_register), offset);
    vm.mem_write(address, vm.reg_index_read(source));
}

pub(crate) fn rti<IO: IOHandle>(_vm: &mut VM<IO>, _command: &Command) {
    unimplemented!("rti command is unused");
}

pub(crate) fn not<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target = command.bit_slice(4, 6) as u8;
    let source = command.bit_slice(7, 9) as u8;
    vm.reg_index_write(target, !vm.reg_index_read(source));
    vm.update_flags(target.into());
}

pub(crate) fn load_indirect<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let pc_offset = sign_extend(command.bit_slice(7, 15), 9);
    let pc = vm.reg_read(RPC);

    let target = command.bit_slice(4, 6) as u8;
    let final_address = vm.mem_read(wrapping_add!(pc, pc_offset));
    vm.reg_index_write(target, vm.mem_read(final_address));
    vm.update_flags(target.into());
}

pub(crate) fn store_indirect<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let source = command.bit_slice(4, 6) as u8;
    let offset = sign_extend(command.bit_slice(7, 15), 9);
    let pc = vm.reg_read(RPC);
    let address = wrapping_add!(pc, offset);
    let final_address = vm.mem_read(address);
    vm.mem_write(final_address, vm.reg_index_read(source));
}

pub(crate) fn jump<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let address_reg = command.bit_slice(7, 9) as u8;
    vm.reg_write(RPC, vm.reg_index_read(address_reg));
}

pub(crate) fn reserved<IO: IOHandle>(_vm: &mut VM<IO>, _command: &Command) {
    unimplemented!("Reserved command is unused");
}

pub(crate) fn load_effective_address<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let target = command.bit_slice(4, 6) as u8;
    let offset = sign_extend(command.bit_slice(7, 15), 9);
    vm.reg_index_write(target, wrapping_add!(vm.reg_read(RPC), offset));
    vm.update_flags(target.into());
}

pub(crate) fn trap<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) {
    let code = command.bit_slice(8, 15) as u8;
    let code = TrapCode::from_int(code);
    match code {
        TrapCode::GetC => handle_trap::getchar(vm),
        TrapCode::Out => handle_trap::trap_out(vm),
        TrapCode::PutS => handle_trap::put_string(vm),
        TrapCode::In => handle_trap::trap_in(vm),
        TrapCode::PutSp => handle_trap::put_byte_string(vm),
        TrapCode::Halt => handle_trap::trap_halt(vm),
    }
}
