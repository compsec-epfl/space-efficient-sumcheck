use crate::hypercube::Hypercube;

// This is like Hypercube, but it only returns the index (which is gray code ordering) and it does not initialize the member itself
#[derive(Debug)]
pub struct HypercubeIndices {
    stop_value: usize,
    value: usize,
}

impl HypercubeIndices {
    pub fn new(num_vars: usize) -> Self {
        Self {
            stop_value: Hypercube::stop_value(num_vars),
            value: 0,
        }
    }
}

impl Iterator for HypercubeIndices {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we reached stop_member
        if self.value >= self.stop_value {
            return None;
        }

        // Increment
        let current_value = self.value;
        self.value = Hypercube::next_gray_code(self.value);

        // Return current index
        Some(current_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::hypercube::HypercubeIndices;

    #[test]
    fn gray_code_indices() {
        // https://docs.rs/gray-codes/latest/gray_codes/struct.GrayCode8.html#examples
        // for n=0, should return empty vec first call, none second call
        let mut hypercube_size_0 = HypercubeIndices::new(0);
        assert_eq!(hypercube_size_0.next().unwrap(), 0);
        // for n=1, should return vec[false] first call, vec[true] second call and None third call
        let mut hypercube_size_1: HypercubeIndices = HypercubeIndices::new(1);
        assert_eq!(hypercube_size_1.next().unwrap(), 0);
        assert_eq!(hypercube_size_1.next().unwrap(), 1);
        assert_eq!(hypercube_size_1.next(), None);
        // so on for n=2
        let mut hypercube_size_2: HypercubeIndices = HypercubeIndices::new(2);
        assert_eq!(hypercube_size_2.next().unwrap(), 0);
        assert_eq!(hypercube_size_2.next().unwrap(), 1);
        assert_eq!(hypercube_size_2.next().unwrap(), 3);
        assert_eq!(hypercube_size_2.next().unwrap(), 2);
        assert_eq!(hypercube_size_2.next(), None);
        // so on for n=3
        let mut hypercube_size_3: HypercubeIndices = HypercubeIndices::new(3);
        assert_eq!(hypercube_size_3.next().unwrap(), 0);
        assert_eq!(hypercube_size_3.next().unwrap(), 1);
        assert_eq!(hypercube_size_3.next().unwrap(), 3);
        assert_eq!(hypercube_size_3.next().unwrap(), 2);
        assert_eq!(hypercube_size_3.next().unwrap(), 6);
        assert_eq!(hypercube_size_3.next().unwrap(), 7);
        assert_eq!(hypercube_size_3.next().unwrap(), 5);
        assert_eq!(hypercube_size_3.next().unwrap(), 4);
        assert_eq!(hypercube_size_3.next(), None);
    }
}