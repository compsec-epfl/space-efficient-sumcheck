use ark_ff::Field;
use ark_std::vec::Vec;

use crate::interpolation::lagrange_polynomial;
use crate::Hypercube;
use crate::Prover;

// the state of the tradeoff prover in the protocol
pub struct TradeoffProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_stages: usize,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
    pub stage_size: usize,
}

impl<F: Field> TradeoffProver<F> {
    pub fn new(evaluations: Vec<F>, num_stages: usize) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        let claimed_evaluation: F = evaluations.iter().sum();
        let num_variables: usize = evaluations.len().ilog2() as usize;
        let stage_size: usize = num_variables / num_stages;
        // return the TradeoffProver instance
        Self {
            claimed_evaluation,
            current_round: 0,
            evaluations,
            num_stages,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
            stage_size,
        }
    }
    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }
    fn precompute_stage_evaluations(&self) -> Vec<F> {
        // define the ranges like so
        let num_vars_b1: usize = self.current_stage() * self.stage_size;
        let num_vars_b2: usize = self.stage_size;
        let num_vars_b3: usize = (self.num_stages - self.current_stage() - 1) * self.stage_size;
        // precompute the evaluations
        let mut precomputed: Vec<F> = vec![F::ZERO; 2_usize.pow(num_vars_b2 as u32)];
        for (index_b1, b1) in Hypercube::<F>::new(num_vars_b1).enumerate() {
            let weight: F = lagrange_polynomial(&b1, &self.verifier_messages[0..b1.len()]).unwrap();
            for index_b2 in 0..2_usize.pow(num_vars_b2 as u32) {
                for index_b3 in 0..2_usize.pow(num_vars_b3 as u32) {
                    let evaluations_index =
                        index_b1 << num_vars_b2 + num_vars_b3 | index_b2 << num_vars_b3 | index_b3;
                    precomputed[index_b2] =
                        precomputed[index_b2] + weight * self.evaluations[evaluations_index];
                }
            }
        }
        return precomputed;
    }
    fn evaluate(&self, precomputed: Vec<F>) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let num_vars_b1 = self.current_stage() * self.stage_size;
        let num_vars_b2_prime = self.current_round - num_vars_b1;
        let num_vars_b2_prime_prime = self.stage_size - num_vars_b2_prime;
        let bitmask: usize = 1 << num_vars_b2_prime_prime - 1;
        for (index_b2_prime, b2_prime) in Hypercube::new(num_vars_b2_prime).enumerate() {
            let weight: F =
                lagrange_polynomial(&b2_prime, &self.verifier_messages[0..b2_prime.len()]).unwrap();

            for index_b2_prime_prime in 0..2_usize.pow(num_vars_b2_prime_prime as u32) {
                let precomputed_index =
                    index_b2_prime << num_vars_b2_prime_prime | index_b2_prime_prime;
                let is_set: bool = (index_b2_prime_prime & bitmask) != 0;
                match is_set {
                    false => sum_0 += precomputed[precomputed_index] * weight,
                    true => sum_1 += precomputed[precomputed_index] * weight,
                }
            }
        }
        return (sum_0, sum_1);
    }
}

impl<F: Field> Prover<F> for TradeoffProver<F> {
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
            // store the verifier message
            self.verifier_messages.push(verifier_message.unwrap());
        }

        let precomputed: Vec<F> = self.precompute_stage_evaluations();

        // // compute the range sum lookup over array of b2 values
        // let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(precomputed.len());
        // let mut running_sum = F::ZERO;
        // for eval in &precomputed {
        //     running_sum += eval;
        //     partial_sums.push(running_sum);
        // }

        // compute the sum
        let evals: (F, F) = self.evaluate(precomputed);

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
    use super::TradeoffProver;
    use crate::unit_test_helpers::{
        run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
    };

    #[test]
    fn sumcheck() {
        run_boolean_sumcheck_test(TradeoffProver::new(test_polynomial(), 1));
        run_basic_sumcheck_test(TradeoffProver::new(test_polynomial(), 1));
        run_boolean_sumcheck_test(TradeoffProver::new(test_polynomial(), 3));
        run_basic_sumcheck_test(TradeoffProver::new(test_polynomial(), 3));
    }
}
