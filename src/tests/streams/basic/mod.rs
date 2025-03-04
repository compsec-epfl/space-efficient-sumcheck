use crate::streams::Stream;
use ark_ff::Field;

/*
 * We want to run sumcheck over some known polynomials, so we can use this
 * stream to pass in a vector containing evaluations of the polynomial
 * from 0..n^2
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
