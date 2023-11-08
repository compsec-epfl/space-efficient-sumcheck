use ark_ff::Field;
use ark_std::vec::Vec;

use super::Prover;

// the state of the time prover in the protocol
pub struct TimeProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> TimeProver<F> {
    pub fn new(evaluations: Vec<F>) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the TimeProver instance
        let claimed_evaluation: F = evaluations.iter().sum();
        let num_variables: usize = evaluations.len().ilog2() as usize;
        Self {
            claimed_evaluation,
            current_round: 0,
            evaluations,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
        }
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
    fn vsbw_evaluate(&self) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let bitmask: usize = 1 << self.num_free_variables() - 1;
        for i in 0..self.evaluations.len() {
            let is_set: bool = (i & bitmask) != 0;
            match is_set {
                false => sum_0 += self.evaluations[i],
                true => sum_1 += self.evaluations[i],
            }
        }
        (sum_0, sum_1)
    }
    fn vsbw_reduce_evaluations(&mut self, verifier_message: F) {
        let half_size: usize = self.evaluations.len() / 2;
        let setbit: usize = 1 << self.num_free_variables(); // we use this to index the second half of the last round's evaluations e.g 001 AND 101
        for i0 in 0..half_size {
            let i1 = i0 | setbit;
            self.evaluations[i0] = self.evaluations[i0] * (F::ONE - verifier_message)
                + self.evaluations[i1] * verifier_message;
        }
        self.evaluations.truncate(half_size);
    }
}

impl<F: Field> Prover<F> for TimeProver<F> {
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
    }
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, reduce the evaluations table
        if self.current_round != 0 {
            // update the evaluations table by absorbing leftmost variable assigned to verifier_message
            self.vsbw_reduce_evaluations(verifier_message.unwrap())
        }

        // evaluate using vsbw
        let evals = self.vsbw_evaluate();

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial
        return Some(evals);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::TimeProver;
    use crate::provers::unit_test_helpers::{
        run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
    };

    #[test]
    fn sumcheck() {
        run_boolean_sumcheck_test(TimeProver::new(test_polynomial()));
        run_basic_sumcheck_test(TimeProver::new(test_polynomial()));
    }
}
