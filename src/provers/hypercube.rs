pub struct Hypercube {
    num_variables: usize,
    current_member: usize,
    stop_member: usize, // stop at this number (exclusive)
}

impl Hypercube {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 2usize.pow(num_variables as u32);
        Self {
            num_variables,
            current_member: 0,
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
        if self.current_member >= self.stop_member {
            return None;
        } else if self.num_variables == 0 {
            self.current_member += 1;
            return Some(vec![]);
        }

        let mut point: Vec<bool> = Vec::with_capacity(self.num_variables);
        for i in (0..self.num_variables).rev() {
            let bit: bool = 0 != (self.current_member >> i) & 1;
            point.push(bit);
        }
        self.current_member += 1;
        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::hypercube::Hypercube;

    #[test]
    fn basic() {
        // small n
        let hypercube_size_0 = Hypercube::new(0);
        let expected_0: Vec<Vec<bool>> = vec![vec![], vec![]];
        for (i, point) in hypercube_size_0.enumerate() {
            assert_eq!(expected_0[i], point);
        }
        let hypercube_size_1 = Hypercube::new(1);
        let expected_1: Vec<Vec<bool>> = vec![vec![false], vec![true]];
        for (i, point) in hypercube_size_1.enumerate() {
            assert_eq!(expected_1[i], point);
        }
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
