// Basically this emulates a Vec<bool> as an iterator wrapped over a usize
#[derive(Clone, Debug, PartialEq)]
pub struct HypercubeMember {
    bit_index: usize,
    num_vars: usize,
    value: usize,
}

impl HypercubeMember {
    pub fn new(num_vars: usize, value: usize) -> Self {
        assert!(num_vars <= std::mem::size_of::<usize>() * 8);
        Self {
            bit_index: num_vars,
            num_vars,
            value,
        }
    }
    pub fn new_from_vec_bool(value: Vec<bool>) -> Self {
        HypercubeMember::new(value.len(), HypercubeMember::usize_from_vec_bool(value))
    }
    pub fn len(&self) -> usize {
        self.num_vars
    }
    // pub fn partition(
    //     h: HypercubeMember,
    //     indices: Vec<usize>,
    // ) -> (HypercubeMember, HypercubeMember) {
    //     assert!(h.len() >= indices.len());
    //     let mut partition_1: Vec<bool> = Vec::with_capacity(h.len() - indices.len());
    //     let mut partition_2: Vec<bool> = Vec::with_capacity(indices.len());
    //     let mut partitioned = 0;
    //     for (index, bit) in h.clone().into_iter().enumerate() {
    //         if partitioned < indices.len() && index == indices[partitioned] {
    //             partition_2.push(bit);
    //             partitioned += 1;
    //         } else {
    //             partition_1.push(bit);
    //         }
    //     }
    //     (
    //         HypercubeMember::new_from_vec_bool(partition_1),
    //         HypercubeMember::new_from_vec_bool(partition_2),
    //     )
    // }
    pub fn usize_from_vec_bool(vec: Vec<bool>) -> usize {
        vec.into_iter()
            .rev()
            .enumerate()
            .fold(0, |acc, (i, bit)| acc | ((bit as usize) << i))
    }
    pub fn elements_at_indices(b: Vec<bool>, indices: Vec<usize>) -> Vec<bool> {
        // checks
        if indices.len() == 0 {
            return vec![];
        }
        assert!(b.len() >= indices.len());
        assert!(b.len() > *indices.last().unwrap());
        // get the indices
        let mut b_prime: Vec<bool> = Vec::with_capacity(indices.len());
        for index in &indices {
            b_prime.push(b[*index]);
        }
        b_prime
    }
    pub fn to_vec_bool(&self) -> Vec<bool> {
        let mut b: Vec<bool> = Vec::with_capacity(self.num_vars);
        for bit_index in (0..self.num_vars).rev() {
            b.push(self.value & (1 << bit_index) != 0);
        }
        b
    }
}

impl Iterator for HypercubeMember {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        // Check if n == 0
        if self.bit_index == 0 {
            return None;
        }
        // Return if value is bit high at bit_index
        self.bit_index = self.bit_index - 1;
        let bit_mask = 1 << self.bit_index;
        Some(self.value & bit_mask != 0)
    }
}

// On each call to next() this gives a HypercubeMember for the value
#[derive(Debug)]
pub struct Hypercube {
    num_vars: usize,
    stop_value: usize,
    value: usize,
}

impl Hypercube {
    pub fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            stop_value: Self::stop_value(num_vars),
            value: 0,
        }
    }
    pub fn stop_value(num_vars: usize) -> usize {
        1 << num_vars // this is exclusive, meaning should stop *before* this value
    }
    pub fn next_gray_code(value: usize) -> usize {
        let mask = match value.count_ones() & 1 == 0 {
            true => 1,
            false => 1 << (value.trailing_zeros() + 1),
        };
        value ^ mask
    }
    pub fn last_gray_code(value: usize) -> usize {
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
        if self.value >= self.stop_value {
            return None;
        }

        // Increment
        let current_value = self.value;
        self.value = Self::next_gray_code(self.value);

        // Return current member
        Some((
            current_value,
            HypercubeMember::new(self.num_vars, current_value),
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
        // for n=0, should return empty vec first call, none second call
        let mut hypercube_size_0 = Hypercube::new(0);
        is_eq(hypercube_size_0.next().unwrap().1, vec![]);
        // for n=1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1: Hypercube = Hypercube::new(1);
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
        // for n=0, should return empty vec first call, none second call
        let mut hypercube_size_0 = Hypercube::new(0);
        assert_eq!(hypercube_size_0.next().unwrap().0, 0);
        // for n=1, should return vec[false] first call, vec[true] second call and None third call
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
    // #[test]
    // fn partition() {
    //     let test_1 = HypercubeMember::new_from_vec_bool(vec![true, false, false, false, false]);
    //     let indices_1 = vec![2, 3];
    //     let result_1 = HypercubeMember::partition(test_1, indices_1);
    //     is_eq(result_1.0, vec![true, false, false]);
    //     is_eq(result_1.1, vec![false, false]);
    //     let test_2 = HypercubeMember::new_from_vec_bool(vec![
    //         false, true, true, false, false, false, false, true,
    //     ]);
    //     let indices_2 = vec![0, 1, 2, 4, 6];
    //     let result_2 = HypercubeMember::partition(test_2, indices_2);
    //     is_eq(result_2.0, vec![false, false, true]);
    //     is_eq(result_2.1, vec![false, true, true, false, false, false]);
    // }
    #[test]
    fn elements_at_indices() {
        let test_1 = vec![true, false, false, false, false];
        let indices_1 = vec![2, 3];
        let result_1 = HypercubeMember::elements_at_indices(test_1, indices_1);
        assert_eq!(result_1, vec![false, false]);
        let test_2 = vec![false, true, true, false, false, false, false, true];
        let indices_2 = vec![0, 1, 2, 4, 6];
        let result_2 = HypercubeMember::elements_at_indices(test_2, indices_2);
        assert_eq!(result_2, vec![false, true, true, false, false]);
    }
    #[test]
    fn vec_bool_to_usize() {
        let test_1 = vec![true, false, false];
        let exp_1 = 4;
        assert_eq!(HypercubeMember::usize_from_vec_bool(test_1), exp_1);
        let test_2 = vec![false, true, true];
        let exp_2 = 3;
        assert_eq!(HypercubeMember::usize_from_vec_bool(test_2), exp_2);
    }
    #[test]
    fn to_vec_bool() {
        let exp_1 = vec![true, false, false, false, false];
        let test_1 = HypercubeMember::new_from_vec_bool(exp_1.clone());
        assert_eq!(exp_1, test_1.to_vec_bool());
        let test_2 = HypercubeMember::new(5, 16);
        assert_eq!(exp_1, test_2.to_vec_bool());

        let exp_2 = vec![false, false, true, false, true];
        let test_3 = HypercubeMember::new_from_vec_bool(exp_2.clone());
        assert_eq!(exp_2, test_3.to_vec_bool());
        let test_4 = HypercubeMember::new(5, 5);
        assert_eq!(exp_2, test_4.to_vec_bool());

        let exp_3 = vec![false, false, true];
        let test_3 = HypercubeMember::new(3, 1);
        assert_eq!(test_3.to_vec_bool(), exp_3);
    }
}
