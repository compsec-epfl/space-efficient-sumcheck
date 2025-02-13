use crate::streams::EvaluationStream;
use ark_ff::Field;

/*
 * We want to run sumcheck over some known polynomials, so we can use this
 * stream to pass in a vector containing evaluations of the polynomial
 * from 0..n^2
 */

#[derive(Debug)]
pub struct BasicEvaluationStream<F: Field> {
    pub evaluations: Vec<F>,
}

impl<F: Field> BasicEvaluationStream<F> {
    pub fn new(evaluations: Vec<F>) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the BasicEvaluationStream instance
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

impl<F: Field> EvaluationStream<F> for BasicEvaluationStream<F> {
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
