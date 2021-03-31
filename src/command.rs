use crate::error::{LC3Error, LC3Result};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Command {
    bytes: u16,
}

impl Command {
    pub(crate) fn new(bytes: u16) -> Self {
        Self { bytes }
    }

    // The op_code is the leftmost 4 bits of the command
    pub(crate) fn op_code(&self) -> LC3Result<u8> {
        Ok(self.bit_slice(0, 3)? as u8)
    }

    pub(crate) fn get_bytes(&self) -> u16 {
        self.bytes
    }

    // Return the bits bitween left and right index (inclusive) as a u16
    // The bits will be rshifted, so the rightmost bit of the output
    // will be the rightmost bit of the u16.
    pub(crate) fn bit_slice(&self, left: u8, right: u8) -> LC3Result<u16> {
        if right > 15 {
            return Err(LC3Error::Internal(format!(
                "Right index for bit_slice exceeded 15. Value: {}",
                right
            )));
        }

        if left > right {
            return Err(LC3Error::Internal(format!(
                "Left ({}) for bit_slice exceeded right ({})",
                left, right
            )));
        }

        let left_mask = 0xFFFF >> left;
        let masked = self.bytes & left_mask;
        let rshift_size = 15 - right;
        Ok(masked >> rshift_size)
    }
}

#[cfg(test)]
mod test {
    use super::Command;

    #[test]
    fn can_read_op_codes() {
        let byte_op_pairs = [
            (0x0000, 0),
            (0x1000, 1),
            (0x2000, 2),
            (0x3000, 3),
            (0x4000, 4),
            (0x5000, 5),
            (0x6000, 6),
            (0x7000, 7),
            (0x8000, 8),
            (0x9000, 9),
            (0xA000, 10),
            (0xB000, 11),
            (0xC000, 12),
            (0xD000, 13),
            (0xE000, 14),
            (0xF000, 15),
        ];

        for (bytes, op_code) in &byte_op_pairs {
            let command = Command::new(*bytes);
            let command_op_code = command.op_code();
            assert_eq!(*op_code, command_op_code.unwrap());
        }
    }
}
