// Basically, this works like Vec<bool> as an iterator wrapped over a usize
#[derive(Debug, PartialEq)]
pub struct HypercubeMember {
    bit_index: usize,
    value: usize,
}

impl HypercubeMember {
    pub fn new(num_vars: usize, value: usize) -> Self {
        Self {
            bit_index: num_vars,
            value,
        }
    }
}

impl Iterator for HypercubeMember {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        // Check if n == 0
        if self.bit_index == 0 {
            return None;
        }
        // Return if the bit of self.value is set at the given index
        self.bit_index = self.bit_index - 1;
        let bit_mask = 1 << self.bit_index;
        Some(self.value & bit_mask != 0)
    }
}

// Basically, on each call to next this gives back a HyperCube member for the current index
#[derive(Debug)]
pub struct Hypercube {
    num_vars: usize,
    stop_index: usize, // stop at this index (exclusive)
    value: usize,
}

impl Hypercube {
    pub fn new(num_vars: usize) -> Self {
        let stop_index: usize = Self::stop_member_from_size(num_vars);
        Self {
            num_vars,
            stop_index,
            value: 0,
        }
    }
    pub fn stop_member_from_size(num_variables: usize) -> usize {
        1 << num_variables
    }
    pub fn next_gray_code(value: usize) -> usize {
        let mask = match value.count_ones() & 1 == 0 {
            true => 1,
            false => 1 << (value.trailing_zeros() + 1),
        };
        value ^ mask
    }
}

impl Iterator for Hypercube {
    type Item = (usize, HypercubeMember);
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we reached stop_member
        if self.value >= self.stop_index {
            return None;
        }

        // Increment
        let current_code = self.value;
        self.value = Self::next_gray_code(self.value);

        // Return current member
        Some((
            current_code,
            HypercubeMember::new(self.num_vars, current_code as usize),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::hypercube::{Hypercube, HypercubeMember};

    fn is_eq(given: HypercubeMember, expected: Vec<bool>) {
        // check each value in the vec
        for (i, (a, b)) in given.zip(expected.clone()).enumerate() {
            assert_eq!(
                a, b,
                "bit at index {} incorrect, should be {:?}",
                i, expected
            );
        }
    }

    #[test]
    fn gray_code_hypercube_members() {
        // https://docs.rs/gray-codes/latest/gray_codes/struct.GrayCode8.html#examples
        // for 0, should return empty vec first call, none second call
        let mut hypercube_size_0 = Hypercube::new(0);
        is_eq(hypercube_size_0.next().unwrap().1, vec![]);
        // for 1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1: Hypercube = Hypercube::new(1);
        println!("{:?}", hypercube_size_1);
        is_eq(hypercube_size_1.next().unwrap().1, vec![false]);
        is_eq(hypercube_size_1.next().unwrap().1, vec![true]);
        assert_eq!(hypercube_size_1.next(), None);
        // so on for n=2
        let mut hypercube_size_2: Hypercube = Hypercube::new(2);
        is_eq(hypercube_size_2.next().unwrap().1, vec![false, false]);
        is_eq(hypercube_size_2.next().unwrap().1, vec![false, true]);
        is_eq(hypercube_size_2.next().unwrap().1, vec![true, true]);
        is_eq(hypercube_size_2.next().unwrap().1, vec![true, false]);
        assert_eq!(hypercube_size_2.next(), None);
        // so on for n=3
        let mut hypercube_size_3: Hypercube = Hypercube::new(3);
        is_eq(
            hypercube_size_3.next().unwrap().1,
            vec![false, false, false],
        );
        is_eq(hypercube_size_3.next().unwrap().1, vec![false, false, true]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![false, true, true]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![false, true, false]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![true, true, false]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![true, true, true]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![true, false, true]);
        is_eq(hypercube_size_3.next().unwrap().1, vec![true, false, false]);
        assert_eq!(hypercube_size_3.next(), None);
    }
    #[test]
    fn gray_code_indices() {
        // https://docs.rs/gray-codes/latest/gray_codes/struct.GrayCode8.html#examples
        // for 0, should return empty vec first call, none second call
        let mut hypercube_size_0 = Hypercube::new(0);
        assert_eq!(hypercube_size_0.next().unwrap().0, 0);
        // for 1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1: Hypercube = Hypercube::new(1);
        assert_eq!(hypercube_size_1.next().unwrap().0, 0);
        assert_eq!(hypercube_size_1.next().unwrap().0, 1);
        assert_eq!(hypercube_size_1.next(), None);
        // so on for n=2
        let mut hypercube_size_2: Hypercube = Hypercube::new(2);
        assert_eq!(hypercube_size_2.next().unwrap().0, 0);
        assert_eq!(hypercube_size_2.next().unwrap().0, 1);
        assert_eq!(hypercube_size_2.next().unwrap().0, 3);
        assert_eq!(hypercube_size_2.next().unwrap().0, 2);
        assert_eq!(hypercube_size_2.next(), None);
        // so on for n=3
        let mut hypercube_size_3: Hypercube = Hypercube::new(3);
        assert_eq!(hypercube_size_3.next().unwrap().0, 0);
        assert_eq!(hypercube_size_3.next().unwrap().0, 1);
        assert_eq!(hypercube_size_3.next().unwrap().0, 3);
        assert_eq!(hypercube_size_3.next().unwrap().0, 2);
        assert_eq!(hypercube_size_3.next().unwrap().0, 6);
        assert_eq!(hypercube_size_3.next().unwrap().0, 7);
        assert_eq!(hypercube_size_3.next().unwrap().0, 5);
        assert_eq!(hypercube_size_3.next().unwrap().0, 4);
        assert_eq!(hypercube_size_3.next(), None);
    }
}
