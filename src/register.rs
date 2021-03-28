const REGISTERS: [Register;11] = [
    Register::RR0,
    Register::RR1,
    Register::RR2,
    Register::RR3,
    Register::RR4,
    Register::RR5,
    Register::RR6,
    Register::RR7,
    Register::RPC,
    Register::RCond,
    Register::RCount

];

pub(crate) const NUM_REGISTERS: usize = REGISTERS.len();

#[derive(Copy, Clone)]
pub(crate) enum Register {
    RR0 = 0,
    RR1 = 1,
    RR2 = 2,
    RR3 = 3,
    RR4 = 4,
    RR5 = 5,
    RR6 = 6,
    RR7 = 7,
    RPC = 8,
    RCond = 9,
    RCount = 10
}

impl Register {
    pub(crate) fn to_u8(&self) -> u8 {
        *self as u8
    }

    #[cfg(test)]
    pub(crate) fn index(&self) -> usize {
        self.to_u8() as usize
    }
}
