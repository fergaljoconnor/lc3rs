use crate::error::{LC3Error, LC3Result};

const OP_CODES: [Op;16] = [
    Op::Br,
    Op::Add,
    Op::Ld,
    Op::St,
    Op::Jsr,
    Op::And,
    Op::Ldr,
    Op::Str,
    Op::Rti,
    Op::Not,
    Op::Ldi,
    Op::Sti,
    Op::Jmp,
    Op::Res,
    Op::Lea,
    Op::Trap,
];

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Op {
    Br,   /* branch */
    Add,  /* add  */
    Ld,   /* load */
    St,   /* store */
    Jsr,  /* jump register */
    And,  /* bitwise and */
    Ldr,  /* load register */
    Str,  /* store register */
    Rti,  /* unused */
    Not,  /* bitwise not */
    Ldi,  /* load indirect */
    Sti,  /* store indirect */
    Jmp,  /* jump */
    Res,  /* reserved (unused) */
    Lea,  /* load effective address */
    Trap, /* execute trap */
}

impl Op {
    pub(crate) fn from_int(op_code: u8) -> LC3Result<Self> {
        if (op_code as usize) < OP_CODES.len() {
            return Ok( OP_CODES[op_code as usize].clone());
        } else {
            let err = LC3Error::BadOpCode{code: op_code};
            Err(err)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Op, OP_CODES};

    #[test]
    fn can_cast_int_to_instruction() {
        for (code, op) in OP_CODES.iter().enumerate() {
            assert_eq!(&Op::from_int(code as u8).unwrap(), op);
        }
    }
}
