use std::marker::PhantomData;

use ark_ff::Field;

pub struct Hypercube<F: Field> {
    num_variables: usize,
    current_member: usize,
    stop_member: usize, // we stop iterating when we reach this number (exclusive)
    _f: PhantomData<F>,
}
pub struct HypercubeChunk<F: Field> {
    hypercube: Hypercube<F>,
    chunk_size: usize,
    current_member: usize,
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

impl<F: Field> HypercubeChunk<F> {
    pub fn new(num_variables: usize, chunk_size: usize) -> Self {
        assert_ne!(0, chunk_size);
        let hypercube: Hypercube<F> = Hypercube::<F>::new(num_variables);
        Self {
            hypercube,
            chunk_size,
            current_member: 0,
            _f: PhantomData,
        }
    } 
    pub fn new_from_hypercube(hypercube: Hypercube<F>, chunk_size: usize) -> Self {
        assert_ne!(0, chunk_size);
        Self {
            hypercube,
            chunk_size,
            current_member: 0,
            _f: PhantomData,
        }
    }
}

impl<F: Field> Iterator for Hypercube<F> {
    type Item = Vec<F>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_member >= self.stop_member {
            return None;
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

impl<F: Field> Iterator for HypercubeChunk<F> {
    type Item = Hypercube<F>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_member >= self.hypercube.stop_member {
            return None;
        }

        let start_member = self.current_member;
        let stop_member = match self.current_member + self.chunk_size >= self.hypercube.stop_member
        {
            true => self.hypercube.stop_member,
            false => self.current_member + self.chunk_size,
        };
        self.current_member += self.chunk_size;
        Some(Hypercube::<F>::new_from_range(
            self.hypercube.num_variables,
            start_member,
            stop_member,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::{
        fields::Fp64,
        fields::{MontBackend, MontConfig},
    };

    use pretty_assertions::assert_eq;

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;

    #[test]
    fn small_n() {
        let hypercube = Hypercube::<TestField>::new(0);
        let points = vec![vec![TestField::ZERO], vec![TestField::ONE]];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }

    #[test]
    fn numerical_order() {
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

    #[test]
    fn chunk() {
        let num_variables = 3;
        for chunk_size in 1..10 {
            let points: Vec<Vec<TestField>> = vec![
                vec![TestField::ZERO, TestField::ZERO, TestField::ZERO],
                vec![TestField::ZERO, TestField::ZERO, TestField::ONE],
                vec![TestField::ZERO, TestField::ONE, TestField::ZERO],
                vec![TestField::ZERO, TestField::ONE, TestField::ONE],
                vec![TestField::ONE, TestField::ZERO, TestField::ZERO],
                vec![TestField::ONE, TestField::ZERO, TestField::ONE],
                vec![TestField::ONE, TestField::ONE, TestField::ZERO],
                vec![TestField::ONE, TestField::ONE, TestField::ONE],
            ];
            let mut points_computed: Vec<Vec<TestField>> = Vec::with_capacity(points.len());
            for hypercube in HypercubeChunk::new(num_variables, chunk_size) {
                for point in hypercube {
                    points_computed.push(point);
                }
            }
            assert_eq!(points, points_computed);
        }
    }
}
