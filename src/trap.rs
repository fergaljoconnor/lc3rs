use crate::error::{LC3Error, LC3Result};

pub(crate) enum TrapCode {
    GetC = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    Out = 0x21,   /* output a character */
    PutS = 0x22,  /* output a word string */
    In = 0x23,    /* get character from keyboard, echoed onto the terminal */
    PutSp = 0x24, /* output a byte string */
    Halt = 0x25,  /* halt the program */
}

impl TrapCode {
    pub(crate) fn from_int(code: u8) -> LC3Result<Self> {
        let code = match code {
            0x20 => Self::GetC,
            0x21 => Self::Out,
            0x22 => Self::PutS,
            0x23 => Self::In,
            0x24 => Self::PutSp,
            0x25 => Self::Halt,
            _ => return Err(LC3Error::BadTrapCode { code }),
        };

        Ok(code)
    }
}
