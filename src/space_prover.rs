use ark_ff::Field;

use crate::interpolation::lagrange_polynomial;
use crate::Hypercube;
use crate::Prover;

// the state of the space prover in the protocol
pub struct SpaceProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> SpaceProver<F> {
    pub fn new(evaluations: Vec<F>) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        let claimed_evaluation: F = evaluations.iter().sum();
        let num_variables: usize = evaluations.len().ilog2() as usize;
        // return the TimeProver instance
        Self {
            claimed_evaluation,
            evaluations,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
            current_round: 0,
            num_variables,
        }
    }
    // instance methods
    fn cty_evaluate(&self) -> (F, F) {
        let mut sum_0: F = F::ZERO;
        let mut sum_1: F = F::ZERO;
        let bitmask: usize = 1 << self.num_free_variables() - 1;
        // iterate in two loops
        let num_vars_outer_loop = self.current_round;
        let num_vars_inner_loop = self.num_variables - num_vars_outer_loop;
        for (index_outer, outer) in Hypercube::<F>::new(num_vars_outer_loop).enumerate() {
            let weight: F = lagrange_polynomial(&outer, &self.verifier_messages).unwrap();
            for index_inner in 0..2_usize.pow(num_vars_inner_loop as u32) {
                let evaluation_index = index_outer << num_vars_inner_loop | index_inner;
                let is_set: bool = (evaluation_index & bitmask) != 0;
                match is_set {
                    false => sum_0 += self.evaluations[evaluation_index] * weight,
                    true => sum_1 += self.evaluations[evaluation_index] * weight,
                }
            }
        }
        (sum_0, sum_1)
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
}

impl<F: Field> Prover<F> for SpaceProver<F> {
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
    }
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to verifier_messages
        if self.current_round != 0 {
            self.verifier_messages.push(verifier_message.unwrap());
        }

        // evaluate using cty
        let evals: (F, F) = self.cty_evaluate();

        // don't forget to increment the round
        self.current_round += 1;

        return Some(evals);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::SpaceProver;
    use crate::unit_test_helpers::{
        run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
    };

    #[test]
    fn sumcheck() {
        run_boolean_sumcheck_test(SpaceProver::new(test_polynomial()));
        run_basic_sumcheck_test(SpaceProver::new(test_polynomial()));
    }
}
