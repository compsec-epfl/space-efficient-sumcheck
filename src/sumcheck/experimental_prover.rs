use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;
use ark_std::vec::Vec;

use crate::sumcheck::Prover;

pub struct ExperimentalProver<F: Field> {
    pub claimed_evaluation: F, // the claimed evaluation of the multilinear polynomial
    pub evaluations: Vec<F>,   // evaluated values of the multilinear polynomial for each input
    pub range_sums: Vec<F>,    // range sums used in the computation
    pub random_challenges: Vec<F>, // random challenges for the protocol
    pub current_round: usize,  // current round of the protocol
    pub num_variables: usize,  // number of variables in the multilinear polynomial
}

impl<F: Field> ExperimentalProver<F> {
    // create new time prover state
    pub fn new(evaluations: Vec<F>) -> Self {
        let num_variables: usize = evaluations.len().ilog2() as usize;
        // compute the range sum lookup
        let mut running_sum = F::ZERO;
        let mut range_sums = Vec::<F>::with_capacity(num_variables);
        for point_eval in &evaluations {
            running_sum += point_eval;
            range_sums.push(running_sum);
        }
        // return ExperimentalProver instance
        Self {
            claimed_evaluation: range_sums[num_variables - 1],
            evaluations,
            range_sums,
            random_challenges: Vec::<F>::with_capacity(num_variables),
            current_round: 0,
            num_variables,
        }
    }
    fn bits_to_index(bits: &[F]) -> usize {
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
    // instance methods
    fn num_free_variables(&self) -> usize {
        if self.num_variables == self.random_challenges.len() {
            return 0;
        }
        return self.num_variables - self.random_challenges.len() - 1;
    }
}

impl<F: Field> Prover<F> for ExperimentalProver<F> {
    // a next-message function using vsbw
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to random_challenges
        if self.current_round != 0 {
            self.random_challenges.push(verifier_message.unwrap());
        }

        // Define the start and end of the range to sum over for g(0)
        let sum_0_start_index = ExperimentalProver::<F>::bits_to_index(
            &[
                self.random_challenges.clone(),
                vec![F::ZERO],
                vec![F::ZERO; self.num_free_variables()],
            ]
            .concat(),
        );
        let sum_0_end_index = ExperimentalProver::<F>::bits_to_index(
            &[
                self.random_challenges.clone(),
                vec![F::ZERO],
                vec![F::ONE; self.num_free_variables()],
            ]
            .concat(),
        );

        // Define the start and end of the range to sum over for g(1)
        let sum_1_start_index = sum_0_end_index; // Start index for g(1) is the same as end index fore g(0)
        let sum_1_end_index = ExperimentalProver::<F>::bits_to_index(
            &[
                self.random_challenges.clone(),
                vec![F::ONE],
                vec![F::ONE; self.num_free_variables()],
            ]
            .concat(),
        );

        // // Compute the sums of evaluations using range lookup
        let sum_0 = self.range_sums[sum_0_end_index]
            - if sum_0_start_index > 0 {
                self.range_sums[sum_0_start_index - 1]
            } else {
                F::ZERO
            };
        let sum_1 = self.range_sums[sum_1_end_index] - self.range_sums[sum_1_start_index];

        // Form a polynomial s.t. g(0) = sum_0 and g(1) = sum_1
        let g: SparsePolynomial<F> =
            SparsePolynomial::from_coefficients_vec(vec![(0, sum_0), (1, sum_1 - sum_0)]);

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial
        Some(g)
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
    }
}

#[cfg(test)]
mod tests {
    use super::ExperimentalProver;
    use crate::sumcheck::unit_test_helpers::{run_boolean_sumcheck_test, test_polynomial};

    #[test]
    fn sumcheck() {
        run_boolean_sumcheck_test(ExperimentalProver::new(test_polynomial()));
    }
}
