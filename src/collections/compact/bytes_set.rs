
pub struct BytesSet {
    storage: [u128;2]
}

impl BytesSet {
    pub fn new() -> Self {
        BytesSet {
            storage: [0; 2]
        }
    }

    #[inline]
    pub fn insert(&mut self, elem: &u8) {
        let (i, shift) = get_i_and_shift(elem);

        self.storage[i] |= 1u128 << shift;
    }

    #[inline]
    pub fn contains(&self, elem: &u8) -> bool {
        let (i, shift) = get_i_and_shift(elem);
        (self.storage[i] & (1u128 << shift)) != 0
    }
}

#[inline]
fn get_i_and_shift(elem: &u8) -> (usize, u8) {
    let i;
    let shift;
    if *elem > 127 {
        shift = elem - 128;
        i = 1;
    } else {
        shift = *elem;
        i = 0;
    }

    (i, shift)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_set_op_works() {
        let mut bs = BytesSet::new();
        for i in 0..255 {
            bs.insert(&i);

            assert!(bs.contains(&i));
            assert!(!bs.contains(&(i+1)));
        }
    }
}
