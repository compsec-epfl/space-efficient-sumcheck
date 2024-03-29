// basically, this emulates a Vec<bool> as an iterator wrapped over a usize
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

// this is an iterator that gives back HypercubeMember ^ above on each call to next
pub struct Hypercube {
    num_variables: usize,
    last_member: Option<usize>,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube {
    pub fn new(num_variables: usize) -> Self {
        let stop_member: usize = Self::stop_member_from_size(num_variables);
        Self {
            num_variables,
            last_member: None,
            stop_member,
        }
    }
    pub fn stop_member_from_size(num_variables: usize) -> usize {
        1 << num_variables
    }
}

impl Iterator for Hypercube {
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

#[cfg(test)]
mod tests {
    use crate::provers::hypercube::{Hypercube, HypercubeMember};

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
        let mut hypercube_size_0 = Hypercube::new(0);
        check_eq(hypercube_size_0.next().unwrap(), vec![]);
        // for 1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1: Hypercube = Hypercube::new(1);
        check_eq(hypercube_size_1.next().unwrap(), vec![false]);
        check_eq(hypercube_size_1.next().unwrap(), vec![true]);
        assert_eq!(hypercube_size_1.next(), None);
        // so on for n=2
        let mut hypercube_size_2: Hypercube = Hypercube::new(2);
        check_eq(hypercube_size_2.next().unwrap(), vec![false, false]);
        check_eq(hypercube_size_2.next().unwrap(), vec![false, true]);
        check_eq(hypercube_size_2.next().unwrap(), vec![true, false]);
        check_eq(hypercube_size_2.next().unwrap(), vec![true, true]);
        assert_eq!(hypercube_size_2.next(), None);
        // so on for n=3
        let mut hypercube_size_3: Hypercube = Hypercube::new(3);
        check_eq(hypercube_size_3.next().unwrap(), vec![false, false, false]);
        check_eq(hypercube_size_3.next().unwrap(), vec![false, false, true]);
        check_eq(hypercube_size_3.next().unwrap(), vec![false, true, false]);
        check_eq(hypercube_size_3.next().unwrap(), vec![false, true, true]);
        check_eq(hypercube_size_3.next().unwrap(), vec![true, false, false]);
        check_eq(hypercube_size_3.next().unwrap(), vec![true, false, true]);
        check_eq(hypercube_size_3.next().unwrap(), vec![true, true, false]);
        check_eq(hypercube_size_3.next().unwrap(), vec![true, true, true]);
        assert_eq!(hypercube_size_3.next(), None);
    }
}
