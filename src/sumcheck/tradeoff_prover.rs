use ark_ff::Field;
use ark_std::vec::Vec;

use crate::sumcheck::Hypercube;
use crate::sumcheck::Prover;

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
    // class methods
    pub fn lagrange_polynomial(x: &[F], w: &[F]) -> Option<F> {
        if x.len() != w.len() {
            None
        } else {
            Some(
                x.to_vec()
                    .iter()
                    .zip(w.iter())
                    .fold(F::ONE, |acc, (&x_i, &w_i)| {
                        acc * (x_i * w_i + (F::ONE - x_i) * (F::ONE - w_i))
                    }),
            )
        }
    }
    fn field_elements_to_index(bits: &[F]) -> usize {
        let mut index: usize = 0;

        // Iterate through the bits from most significant to least significant
        for &bit in bits {
            // Shift the index to the left by 1 bit position
            index <<= 1;

            // If the current bit is 1, set the least significant bit of the index to 1
            if bit == F::ONE {
                index |= 1;
            }
        }

        index
    }
    pub fn new(evaluations: Vec<F>, num_stages: usize) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the TradeoffProver instance
        let claimed_evaluation: F = evaluations.iter().sum();
        let num_variables: usize = (evaluations.len() as f64).log2() as usize;
        let stage_size: usize = num_variables / num_stages;
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

        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let current_stage = self.current_round / self.stage_size;
        let mut precomputed: Vec<F> = vec![F::ZERO; 2_usize.pow(self.stage_size as u32)];
        for b1 in Hypercube::<F>::new(current_stage * self.stage_size) {
            let weight: F =
                TradeoffProver::lagrange_polynomial(&b1, &self.verifier_messages[0..b1.len()])
                    .unwrap();
            for b2 in Hypercube::<F>::new(self.stage_size) {
                let b2_index = TradeoffProver::field_elements_to_index(&b2);
                for b3 in
                    Hypercube::<F>::new((self.num_stages - current_stage - 1) * self.stage_size)
                {
                    let f_index = TradeoffProver::field_elements_to_index(
                        &[b1.clone(), b2.clone(), b3.clone()].concat(),
                    );
                    precomputed[b2_index] =
                        precomputed[b2_index] + weight * self.evaluations[f_index];
                }
            }
        }

        // compute the range sum lookup over array of b2 values
        let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(precomputed.len());
        let mut running_sum = F::ZERO;
        for eval in &precomputed {
            running_sum += eval;
            partial_sums.push(running_sum);
        }

        // compute the sum
        let j_prime = self.current_round - (current_stage * self.stage_size);
        for b2_prime in Hypercube::new(j_prime) {
            let weight: F =
                TradeoffProver::lagrange_polynomial(&b2_prime, &self.verifier_messages[0..j_prime])
                    .unwrap();
            for b2_prime_prime in Hypercube::<F>::new(self.stage_size - j_prime) {
                let bitmask: usize = 1 << b2_prime_prime.len() - 1;
                let b2_prime_prime_index: usize = TradeoffProver::field_elements_to_index(
                    &[b2_prime.clone(), b2_prime_prime.clone()].concat(),
                );
                let is_set: bool = (b2_prime_prime_index & bitmask) != 0;
                println!(
                    "prime prime index: {}, bitmask: {}, is_set: {}",
                    b2_prime_prime_index, bitmask, is_set
                );
                match is_set {
                    false => sum_0 += precomputed[b2_prime_prime_index] * weight,
                    true => sum_1 += precomputed[b2_prime_prime_index] * weight,
                }
            }
        }

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial
        return Some((sum_0, sum_1));
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::TradeoffProver;
    use crate::sumcheck::unit_test_helpers::{
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
