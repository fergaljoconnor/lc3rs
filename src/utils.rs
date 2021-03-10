// Take the bit bit_count bits from the left and set everything to the left
// of it to the same value as that bit.
pub(crate) fn sign_extend(x: u16, bit_count: u8) -> u16 {
    // Check if the bit at index 16 - bit_count is set
    if ((x >> (bit_count - 1)) & 1) != 0 {
        // Set everything to the left to one
        // Using bit_count instead of bit_count - 1 here is technically
        // correct, since the leftmost bit is already how we want it but this
        // causes bit shift overflow when the bit count is at 16 so this is the
        // neatest way to express it safely.
        return x | (0xFFFF << (bit_count - 1));
    } else {
        // Set everything to the left to zero
        // Probably redundant (they should be zero if the bit count is
        // accurate), but best to be safe.
        return x & !(0xFFFF << (bit_count - 1));
    }
}

// Wrapping gives us the wrapping behavior we need in debug mode
// For more see the documentation of Wrapping:
// https://doc.rust-lang.org/std/num/struct.Wrapping.html
#[macro_export]
macro_rules! wrapping_add {
    ($left:expr, $right: expr) => {
        {use std::num::Wrapping;
        (Wrapping($left) + Wrapping($right)).0}
    };
}

fn reverse_endianness(bytes:u16) -> u16 {
    (bytes >> 8) | (bytes << 8)
}

#[cfg(test)]
mod test {
    use super::sign_extend;

    #[test]
    fn can_sign_extend() {
        // First test the case where the leftmost bit is one
        // Each tuple is (bitcount, input, expected_output)
        let test_bitcount_input_output: Vec<(u8, u16, u16)> = (0u16..16)
            .map(|x| ((x + 1) as u8, 1 << x, 0xFFFF << x))
            .collect();

        for (bit_count, input, expected) in test_bitcount_input_output {
            let extended = sign_extend(input, bit_count);
            assert_eq!(extended, expected);
        }

        // Next test the case where the leftmost bit is zero
        // The bit twiddling here probably deserves a little explanation.
        // If you compare with the map for the case with one on the left above
        // you'll see the bit count is the same, and the input and expected
        // output are the bitwise negated versions of the input and output
        // versions for that case. Basically, to make sure the function
        // is doing its job, we create an integer where the only bit set
        // to zero is the one that determines the sign extension. Above,
        // we wanted an input where this bit was the only bit set to one,
        // hence the negation.
        let test_bitcount_input_output: Vec<(u8, u16, u16)> = (0u16..16)
            .map(|x| ((x + 1) as u8, !(1 << x), !(0xFFFF << x)))
            .collect();

        for (bit_count, input, expected) in test_bitcount_input_output {
            let extended = sign_extend(input, bit_count);
            assert_eq!(extended, expected);
        }
    }
}
