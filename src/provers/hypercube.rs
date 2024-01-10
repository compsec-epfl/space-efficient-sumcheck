pub struct Hypercube {
    num_variables: usize,
    last_member: Option<usize>,
    last_point: Option<Vec<bool>>,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 2usize.pow(num_variables as u32);
        let current_point: Vec<bool> = vec![false; num_variables];
        Self {
            num_variables,
            last_member: None,
            last_point: None,
            stop_member,
        }
    }
    pub fn pow2(num_variables: usize) -> usize {
        2usize.pow(num_variables as u32)
    }
}

impl Iterator for Hypercube {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        // check if this is first iteration
        if self.last_member == None {
            self.last_member = Some(0);
            self.last_point = Some(vec![false; self.num_variables]);
            return self.last_point;
        }
        // check if we've finished iterating
        let next_member = self.last_member.unwrap() + 1;
        if next_member >= self.stop_member {
            return None;
        }
        // check for special case n=0, return vec![] once and None after
        if self.num_variables == 0 {
            self.current_member += 1;
            return Some(vec![]);
        }
        // we're gonna return whatevers already set to current_point
        let last_point = self.current_point.clone();
        // we want the bit diff of current and next
        let next_member = self.current_member + 1;
        let bit_diff = self.current_member ^ next_member;
        // this tells us all the most significant bits that are already shared that we don't need to touch
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;
        // iterate up to this prefix setting bits to the value of next_member
        println!("last_point: {:?}, low_index_of_prefix: {}", last_point, low_index_of_prefix);
        for bit_index in (0..low_index_of_prefix).rev() {
            let target_bit: bool = (next_member & (1 << bit_index)) != 0;
            self.current_point[bit_index] = target_bit;
        }
        // don't forget to increment current member
        self.current_member += 1;
        Some(last_point)
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::hypercube::Hypercube;

    #[test]
    fn basic() {
        // for 0, should return empty vec first call, none second call
        let mut hypercube_size_0 = Hypercube::new(0);
        assert_eq!(hypercube_size_0.next().unwrap(), vec![]);
        assert_eq!(hypercube_size_0.next(), None);
        // for 1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1 = Hypercube::new(1);
        assert_eq!(hypercube_size_1.next().unwrap(), vec![false]);
        assert_eq!(hypercube_size_1.next().unwrap(), vec![true]);
        assert_eq!(hypercube_size_1.next(), None);
        // so on for n=2
        let hypercube_size_2 = Hypercube::new(2);
        let expected_2: Vec<Vec<bool>> = vec![
            vec![false, false],
            vec![false, true],
            vec![true, false],
            vec![true, true],
        ];
        for (i, point) in hypercube_size_2.enumerate() {
            assert_eq!(expected_2[i], point);
        }
        // so on for n=3
        let hypercube = Hypercube::new(3);
        let points = vec![
            vec![false, false, false],
            vec![false, false, true],
            vec![false, true, false],
            vec![false, true, true],
            vec![true, false, false],
            vec![true, false, true],
            vec![true, true, false],
            vec![true, true, true],
        ];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }
}
