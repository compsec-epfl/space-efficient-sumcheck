pub struct Hypercube {
    num_variables: usize,
    last_member: Option<usize>,
    last_point: Option<Vec<bool>>,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 2usize.pow(num_variables as u32);
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
        // a) check if this is first iteration
        if self.last_member == None {
            self.last_member = Some(0);
            self.last_point = Some(vec![false; self.num_variables]);
            return self.last_point.clone();
        }
        // b) check if in last iteration we finished iterating
        let next_member = self.last_member.unwrap() + 1;
        if next_member >= self.stop_member {
            return None;
        }
        // c) everything else, first get bit diff
        let bit_diff = self.last_member.unwrap() ^ next_member;
        // determine the shared prefix of most significant bits
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;
        // iterate up to this prefix setting bits correctly, half of the time this is only one bit!
        let mut last_point = self.last_point.clone().unwrap();
        for bit_index in (0..low_index_of_prefix).rev() {
            let target_bit: bool = (next_member & (1 << bit_index)) != 0;
            last_point[self.num_variables - bit_index - 1] = target_bit;
        }
        // don't forget to increment current member
        self.last_member = Some(next_member);
        self.last_point = Some(last_point);
        self.last_point.clone()
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
        let mut hypercube_size_2 = Hypercube::new(2);
        assert_eq!(hypercube_size_2.next().unwrap(), vec![false, false]);
        assert_eq!(hypercube_size_2.next().unwrap(), vec![false, true]);
        assert_eq!(hypercube_size_2.next().unwrap(), vec![true, false]);
        assert_eq!(hypercube_size_2.next().unwrap(), vec![true, true]);
        assert_eq!(hypercube_size_2.next(), None);
        // so on for n=3
        let mut hypercube_size_3 = Hypercube::new(3);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![false, false, false]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![false, false, true]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![false, true, false]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![false, true, true]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![true, false, false]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![true, false, true]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![true, true, false]);
        assert_eq!(hypercube_size_3.next().unwrap(), vec![true, true, true]);
        assert_eq!(hypercube_size_3.next(), None);
    }
}
