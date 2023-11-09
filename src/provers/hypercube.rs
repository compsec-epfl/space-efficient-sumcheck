use ark_ff::Field;
use std::marker::PhantomData;

pub struct Hypercube<F: Field> {
    num_variables: usize,
    current_member: usize,
    stop_member: usize, // stop at this number (exclusive)
    _f: PhantomData<F>,
}

impl<F: Field> Hypercube<F> {
    pub fn new(num_variables: usize) -> Self {
        let stop_member = 2usize.pow(num_variables as u32);
        Self {
            num_variables,
            current_member: 0,
            stop_member,
            _f: PhantomData,
        }
    }
    pub fn new_from_range(num_variables: usize, current_member: usize, stop_member: usize) -> Self {
        Self {
            num_variables,
            current_member,
            stop_member,
            _f: PhantomData,
        }
    }
}

impl<F: Field> Iterator for Hypercube<F> {
    type Item = Vec<F>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_member >= self.stop_member {
            return None;
        } else if self.num_variables == 0 {
            self.current_member += 1;
            return Some(vec![]);
        }

        let point_binary_str = format!(
            "{:0width$b}",
            self.current_member,
            width = self.num_variables
        );
        let point: Vec<F> = point_binary_str
            .chars()
            .map(|c| if c == '0' { F::ZERO } else { F::ONE })
            .collect();

        self.current_member += 1;
        Some(point)
    }
}

#[cfg(test)]
mod tests {
    use super::Field;
    use crate::provers::{test_utilities::TestField, Hypercube};

    #[test]
    fn basic() {
        // small n
        let hypercube_size_0 = Hypercube::<TestField>::new(0);
        let expected_0: Vec<Vec<TestField>> = vec![vec![], vec![]];
        for (i, point) in hypercube_size_0.enumerate() {
            assert_eq!(expected_0[i], point);
        }
        let hypercube_size_1 = Hypercube::<TestField>::new(1);
        let expected_1: Vec<Vec<TestField>> = vec![vec![TestField::ZERO], vec![TestField::ONE]];
        for (i, point) in hypercube_size_1.enumerate() {
            assert_eq!(expected_1[i], point);
        }
        let hypercube_size_2 = Hypercube::<TestField>::new(2);
        let expected_2: Vec<Vec<TestField>> = vec![
            vec![TestField::ZERO, TestField::ZERO],
            vec![TestField::ZERO, TestField::ONE],
            vec![TestField::ONE, TestField::ZERO],
            vec![TestField::ONE, TestField::ONE],
        ];
        for (i, point) in hypercube_size_2.enumerate() {
            assert_eq!(expected_2[i], point);
        }
        let hypercube = Hypercube::<TestField>::new(3);
        let points = vec![
            vec![TestField::ZERO, TestField::ZERO, TestField::ZERO],
            vec![TestField::ZERO, TestField::ZERO, TestField::ONE],
            vec![TestField::ZERO, TestField::ONE, TestField::ZERO],
            vec![TestField::ZERO, TestField::ONE, TestField::ONE],
            vec![TestField::ONE, TestField::ZERO, TestField::ZERO],
            vec![TestField::ONE, TestField::ZERO, TestField::ONE],
            vec![TestField::ONE, TestField::ONE, TestField::ZERO],
            vec![TestField::ONE, TestField::ONE, TestField::ONE],
        ];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }
}
