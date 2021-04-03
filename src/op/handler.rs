use super::trap_handler as handle_trap;
use crate::command::Command;
use crate::error::{LC3Error, LC3Result};
use crate::io::IOHandle;
use crate::register::Register::{RCond, RPC, RR7};
use crate::trap::TrapCode;
use crate::utils::sign_extend;
use crate::vm::VM;
use crate::wrapping_add;

pub(crate) fn branch<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let test_flag = command.bit_slice(4, 6)?;
    let flags = vm.reg_read(RCond);
    let will_branch = (flags & test_flag) != 0;

    if will_branch {
        let offset = sign_extend(command.bit_slice(7, 15)?, 9);
        let new_pc = wrapping_add!(vm.reg_read(RPC), offset);
        vm.reg_write(RPC, new_pc);
    };

    Ok(())
}

pub(crate) fn add<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target_reg = command.bit_slice(4, 6)? as u8;
    let left = vm.reg_index_read(command.bit_slice(7, 9)? as u8);
    let immediate = command.bit_slice(10, 10)? == 1;

    let right = if immediate {
        sign_extend(command.bit_slice(11, 15)?, 5)
    } else {
        vm.reg_index_read(command.bit_slice(13, 15)? as u8)
    };

    vm.reg_index_write(target_reg, wrapping_add!(left, right));
    vm.update_flags(target_reg as usize);

    Ok(())
}

pub(crate) fn load<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target_reg = command.bit_slice(4, 6)? as u8;
    let offset = sign_extend(command.bit_slice(7, 15)?, 9);
    let pc = vm.reg_read(RPC);
    let address = wrapping_add!(pc, offset);
    let val = vm.mem_read(address)?;
    vm.reg_index_write(target_reg, val);
    vm.update_flags(target_reg.into());

    Ok(())
}

pub(crate) fn store<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let source = command.bit_slice(4, 6)? as u8;
    let offset = sign_extend(command.bit_slice(7, 15)?, 9);
    let target = wrapping_add!(vm.reg_read(RPC), offset);
    let val = vm.reg_index_read(source);
    vm.mem_write(target, val);

    Ok(())
}

pub(crate) fn jump_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    // Save program counter
    let pc = vm.reg_read(RPC);
    vm.reg_write(RR7, pc);

    let offset_mode = command.bit_slice(4, 4)? == 1;

    let destination = if offset_mode {
        let offset = sign_extend(command.bit_slice(5, 15)?, 11);
        wrapping_add!(vm.reg_read(RPC), offset)
    } else {
        let source_reg = command.bit_slice(7, 9)? as u8;
        vm.reg_index_read(source_reg)
    };

    vm.reg_write(RPC, destination);

    Ok(())
}

pub(crate) fn and<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target_reg = command.bit_slice(4, 6)? as u8;
    let left = vm.reg_index_read(command.bit_slice(7, 9)? as u8);
    let immediate = command.bit_slice(10, 10)? == 1;

    let right = if immediate {
        sign_extend(command.bit_slice(11, 15)?, 5)
    } else {
        vm.reg_index_read(command.bit_slice(13, 15)? as u8)
    };

    vm.reg_index_write(target_reg, left & right);
    vm.update_flags(target_reg.into());

    Ok(())
}

pub(crate) fn load_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target = command.bit_slice(4, 6)? as u8;
    let base = command.bit_slice(7, 9)? as u8;
    let offset = sign_extend(command.bit_slice(10, 15)?, 6);
    let address = wrapping_add!(vm.reg_index_read(base), offset);
    let val = vm.mem_read(address)?;
    vm.reg_index_write(target, val);
    vm.update_flags(target.into());

    Ok(())
}

pub(crate) fn store_register<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let source = command.bit_slice(4, 6)? as u8;
    let base_register = command.bit_slice(7, 9)? as u8;
    let offset = sign_extend(command.bit_slice(10, 15)?, 6);
    let address = wrapping_add!(vm.reg_index_read(base_register), offset);
    let val = vm.reg_index_read(source);
    vm.mem_write(address, val);

    Ok(())
}

pub(crate) fn rti<IO: IOHandle>(_vm: &mut VM<IO>, _command: &Command) -> LC3Result<()> {
    Err(LC3Error::Internal(
        "Attempt to execute unimplemented op code".to_string(),
    ))
}

pub(crate) fn not<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target = command.bit_slice(4, 6)? as u8;
    let source = command.bit_slice(7, 9)? as u8;
    let negated = !vm.reg_index_read(source);
    vm.reg_index_write(target, negated);
    vm.update_flags(target.into());

    Ok(())
}

pub(crate) fn load_indirect<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let pc_offset = sign_extend(command.bit_slice(7, 15)?, 9);
    let pc = vm.reg_read(RPC);

    let target = command.bit_slice(4, 6)? as u8;
    let final_address = vm.mem_read(wrapping_add!(pc, pc_offset))?;
    let val = vm.mem_read(final_address)?;

    vm.reg_index_write(target, val);
    vm.update_flags(target.into());

    Ok(())
}

pub(crate) fn store_indirect<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let source = command.bit_slice(4, 6)? as u8;
    let offset = sign_extend(command.bit_slice(7, 15)?, 9);
    let pc = vm.reg_read(RPC);
    let address = wrapping_add!(pc, offset);
    let final_address = vm.mem_read(address)?;
    let val = vm.reg_index_read(source);
    vm.mem_write(final_address, val);

    Ok(())
}

pub(crate) fn jump<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let address_reg = command.bit_slice(7, 9)? as u8;
    let address = vm.reg_index_read(address_reg);
    vm.reg_write(RPC, address);

    Ok(())
}

pub(crate) fn reserved<IO: IOHandle>(_vm: &mut VM<IO>, _command: &Command) -> LC3Result<()> {
    Err(LC3Error::Internal(
        "Attempt to execute unimplemented op code".to_string(),
    ))
}

pub(crate) fn load_effective_address<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let target = command.bit_slice(4, 6)? as u8;
    let offset = sign_extend(command.bit_slice(7, 15)?, 9);
    let effective_address = wrapping_add!(vm.reg_read(RPC), offset);
    vm.reg_index_write(target, effective_address);
    vm.update_flags(target.into());

    Ok(())
}

pub(crate) fn trap<IO: IOHandle>(vm: &mut VM<IO>, command: &Command) -> LC3Result<()> {
    let code = command.bit_slice(8, 15)? as u8;
    let code = TrapCode::from_int(code);
    match code? {
        TrapCode::GetC => handle_trap::getchar(vm)?,
        TrapCode::Out => handle_trap::trap_out(vm)?,
        TrapCode::PutS => handle_trap::put_string(vm)?,
        TrapCode::In => handle_trap::trap_in(vm)?,
        TrapCode::PutSp => handle_trap::put_byte_string(vm)?,
        TrapCode::Halt => handle_trap::trap_halt(vm)?,
    };

    Ok(())
}
