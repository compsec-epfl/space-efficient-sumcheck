use crate::streams::Stream;
use ark_ff::Field;

/*
 * It's totally reasonable to use this when the evaluations table
 * fits in memory (and yes, it's not so much a stream in this case)
 */

#[derive(Debug)]
pub struct MemoryStream<F: Field> {
    pub evaluations: Vec<F>,
}

impl<F: Field> MemoryStream<F> {
    pub fn new(evaluations: Vec<F>) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the MemoryStream instance
        Self { evaluations }
    }
    pub fn vec_of_field_to_usize(vec: Vec<F>) -> usize {
        // Reverse the vector to start from the least significant bit
        let reversed_vec: Vec<F> = vec.into_iter().rev().collect();

        // Calculate the decimal value
        let decimal_value: usize = reversed_vec
            .iter()
            .enumerate()
            .filter(|(_, &bit)| bit == F::ONE)
            .map(|(i, _)| 2usize.pow(i as u32))
            .sum();

        decimal_value
    }
}

impl<F: Field> Stream<F> for MemoryStream<F> {
    fn claim(&self) -> F {
        self.evaluations.iter().sum()
    }
    fn evaluation(&self, point: usize) -> F {
        self.evaluations[point]
    }
    fn num_variables(&self) -> usize {
        self.evaluations.len().ilog2() as usize
    }
}
