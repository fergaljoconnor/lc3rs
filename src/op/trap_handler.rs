use crate::error::LC3Result;
use crate::io::{IOHandle};
use crate::register::Register::{RR0};
use crate::vm::VM;
use crate::wrapping_add;

pub(crate) fn getchar<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    let ch = vm.getchar() as u16;
    vm.reg_write(RR0, ch);
    Ok(())
}

pub(crate) fn trap_out<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    let ch = vm.reg_read(RR0) as u8 as char;
    vm.putchar(ch);
    Ok(())
}

pub(crate) fn put_string<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    let mut next_address = vm.reg_read(RR0);
    loop {
        let value = vm.mem_read(next_address)?;
        if value != 0 {
            vm.putchar(value as u8 as char);
        } else {
            break;
        }
        next_address += 1;
    };
    Ok(())
}

pub(crate) fn trap_in<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    // TODO: Swap println out for something using the io handle, otherwise
    // this first part isn't testable.
    println!("Enter a character: ");
    let ch = vm.getchar();
    vm.reg_write(RR0, ch as u16);
    vm.putchar(ch);
    Ok(())
}

pub(crate) fn put_byte_string<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    let mut next_address = vm.reg_read(RR0);
    // TODO: Might be a nicer way to express this loop. Feels a bit close
    // to "while true"
    'outer: loop {
        let raw_value = vm.mem_read(next_address)?;
        let left = raw_value as u8;
        let right = (raw_value >> 8) as u8;

        for value in &[left, right] {
            if *value != 0 {
                vm.putchar(*value as char);
            } else {
                break 'outer;
            }
        }
        next_address = wrapping_add!(next_address, 1);
    }

    Ok(())
}

pub(crate) fn trap_halt<IO: IOHandle>(vm: &mut VM<IO>) -> LC3Result<()> {
    vm.set_running(false);
    Ok(())
}
