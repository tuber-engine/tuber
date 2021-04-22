pub trait BitSet {
    /// Sets a bit
    fn set_bit(&mut self, bit: usize);

    /// Returns the value of a bit
    fn bit(&self, bit: usize) -> bool;
}

impl BitSet for [u64] {
    fn set_bit(&mut self, bit: usize) {
        let cell = bit / 64;
        let remainder = bit % 64;
        self[cell] = self[cell] | (1 << remainder);
    }

    fn bit(&self, bit: usize) -> bool {
        let cell = bit / 64;
        let remainder = bit % 64;
        (self[cell] & (1 << remainder)) != 0
    }
}

impl BitSet for u64 {
    fn set_bit(&mut self, bit: usize) {
        *self = *self | (1 << bit);
    }

    fn bit(&self, bit: usize) -> bool {
        (*self & (1 << bit)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_bit_u64() {
        let mut bitset = 0u64;
        bitset.set_bit(1);

        assert_eq!(bitset, 2u64)
    }

    #[test]
    fn bit_u64() {
        let mut bitset = 0u64;
        bitset.set_bit(0);
        bitset.set_bit(2);

        assert_eq!(bitset.bit(2), true);
        assert_eq!(bitset, 5u64);
    }

    #[test]
    fn set_bit_u64_array() {
        let mut bitset = [0u64; 1024];
        bitset.set_bit(64);
        assert_eq!(bitset[1], 1);
    }

    #[test]
    fn bit_u64_array() {
        let mut bitset = [0u64; 1024];
        bitset.set_bit(66);
        bitset.set_bit(2);
        assert_eq!(bitset.bit(66), true);
        assert_eq!(bitset.bit(2), true);
    }
}
