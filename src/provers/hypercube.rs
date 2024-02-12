// this is basically a Vec<bool>, but we only need it for sizes < ~35 variables
#[derive(Debug, PartialEq)]
pub struct HypercubeMember {
    last_index: usize,
    value: usize,
}

impl HypercubeMember {
    pub fn new(num_variables: usize, value: usize) -> Self {
        Self {
            last_index: num_variables,
            value,
        }
    }
}

impl Iterator for HypercubeMember {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        // Check if in the last iteration we finish iterating
        if self.last_index == 0 {
            return None;
        }
        // Compute if the bit of self.value is set at current_index
        let current_index: usize = self.last_index - 1;
        let is_set_at_index: bool = self.value & (1 << current_index) != 0;
        // Increment
        self.last_index = current_index;
        // Return whether the bit of value at current_index is set
        Some(is_set_at_index)
    }
}

pub struct Hypercube2 {
    num_variables: usize,
    last_member: Option<usize>,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube2 {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 1 << num_variables;
        Self {
            num_variables,
            last_member: None,
            stop_member,
        }
    }
    pub fn pow2(num_variables: usize) -> usize {
        1 << num_variables
    }
}

impl Iterator for Hypercube2 {
    type Item = HypercubeMember;
    fn next(&mut self) -> Option<Self::Item> {
        // a) Check if this is the first iteration
        if self.last_member == None {
            // Initialize last member and last point
            self.last_member = Some(0);
            // Return a member for this point
            return Some(HypercubeMember::new(
                self.num_variables,
                self.last_member.unwrap(),
            ));
        }

        // b) Check if in the last iteration we finished iterating
        let next_member = self.last_member.unwrap() + 1;
        if next_member >= self.stop_member {
            return None;
        }

        // c) Everything else, just increment
        self.last_member = Some(next_member);

        // return the member
        Some(HypercubeMember::new(
            self.num_variables,
            self.last_member.unwrap(),
        ))
    }
}

pub struct Hypercube {
    num_variables: usize,
    last_member: Option<usize>,
    last_point: Option<Vec<bool>>,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 1 << num_variables;
        Self {
            num_variables,
            last_member: None,
            last_point: None,
            stop_member,
        }
    }
    pub fn pow2(num_variables: usize) -> usize {
        1 << num_variables
    }
}

impl Iterator for Hypercube {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        // a) Check if this is the first iteration
        if self.last_member == None {
            // Initialize last member and last point
            self.last_member = Some(0);
            self.last_point = Some(vec![false; self.num_variables]);
            // Return the cloned last point
            return self.last_point.clone();
        }

        // b) Check if in the last iteration we finished iterating
        let next_member = self.last_member.unwrap() + 1;
        if next_member >= self.stop_member {
            return None;
        }

        // c) Everything else, first get bit diff
        let bit_diff = self.last_member.unwrap() ^ next_member;

        //   Determine the shared prefix of the most significant bits
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;

        //   Iterate up to this prefix, setting bits correctly (half of the time this is only one bit!)
        let mut last_point = self.last_point.clone().unwrap();
        for bit_index in (0..low_index_of_prefix).rev() {
            let target_bit: bool = (next_member & (1 << bit_index)) != 0;
            last_point[self.num_variables - bit_index - 1] = target_bit;
        }

        //   Don't forget to increment the current member
        self.last_member = Some(next_member);
        self.last_point = Some(last_point);

        //   Return the cloned last point
        self.last_point.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::hypercube::{Hypercube, Hypercube2, HypercubeMember};

    fn check_eq(given: HypercubeMember, expected: Vec<bool>) {
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
    fn basic() {
        // for 0, should return empty vec first call, none second call
        let mut hypercube2_size_0 = Hypercube2::new(0);
        check_eq(hypercube2_size_0.next().unwrap(), vec![]);
        // for 1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube2_size_1: Hypercube2 = Hypercube2::new(1);
        check_eq(hypercube2_size_1.next().unwrap(), vec![false]);
        check_eq(hypercube2_size_1.next().unwrap(), vec![true]);
        assert_eq!(hypercube2_size_1.next(), None);
        // so on for n=2
        let mut hypercube2_size_2: Hypercube2 = Hypercube2::new(2);
        check_eq(hypercube2_size_2.next().unwrap(), vec![false, false]);
        check_eq(hypercube2_size_2.next().unwrap(), vec![false, true]);
        check_eq(hypercube2_size_2.next().unwrap(), vec![true, false]);
        check_eq(hypercube2_size_2.next().unwrap(), vec![true, true]);
        assert_eq!(hypercube2_size_2.next(), None);
        // so on for n=3
        let mut hypercube2_size_3: Hypercube2 = Hypercube2::new(3);
        check_eq(hypercube2_size_3.next().unwrap(), vec![false, false, false]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![false, false, true]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![false, true, false]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![false, true, true]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![true, false, false]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![true, false, true]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![true, true, false]);
        check_eq(hypercube2_size_3.next().unwrap(), vec![true, true, true]);
        assert_eq!(hypercube2_size_3.next(), None);
    }
}
