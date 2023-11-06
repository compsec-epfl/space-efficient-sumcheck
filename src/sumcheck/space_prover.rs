use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Bitcube;
use crate::sumcheck::Prover;

// the state of the space prover in the protocol
pub struct SpaceProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> SpaceProver<F> {
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
        // iterate over two vectors of bits
        for input_start in Bitcube::new(self.current_round) {
            // need a vec of field elements for each outer loop
            let input_start_field_elements: Vec<F> = input_start
                .iter()
                .map(|bit: &bool| -> F {
                    match *bit {
                        false => F::ZERO,
                        true => F::ONE,
                    }
                })
                .collect();
            // compute the lagrange_polynomial for each iteration with all available verifier messages
            let weight: F = SpaceProver::lagrange_polynomial(
                &input_start_field_elements,
                &self.verifier_messages,
            )
            .unwrap();
            for input_end in Bitcube::new(self.num_variables - input_start.len()) {
                // convert the full bitvector into a scalar index and use this to grab the evaluation
                let index: usize = [input_start.clone(), input_end.clone()]
                    .concat()
                    .iter()
                    .fold((|| 0)(), |index: usize, bit: &bool| {
                        (index << 1)
                            | match *bit {
                                false => 0,
                                true => 1,
                            }
                    });
                let evaluation: F = self.evaluations[index];
                // decide which sum this belongs to
                let is_set: bool = (index & bitmask) != 0;
                match is_set {
                    false => sum_0 += evaluation * weight,
                    true => sum_1 += evaluation * weight,
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
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to verifier_messages
        if self.current_round != 0 {
            self.verifier_messages.push(verifier_message.unwrap());
        }

        // evaluate using cty
        let (sum_0, sum_1) = self.cty_evaluate();

        // form a polynomial that s.t. g_round(0) = sum_0, g_round(1) = sum_1
        let g: SparsePolynomial<F> =
            SparsePolynomial::<F>::from_coefficients_vec(vec![(0, sum_0), (1, -sum_0 + sum_1)]);

        // don't forget to increment the round
        self.current_round += 1;

        return Some(g);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::SpaceProver;
    use crate::sumcheck::unit_test_helpers::{
        run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
    };

    #[test]
    fn sumcheck() {
        run_boolean_sumcheck_test(SpaceProver::new(test_polynomial()));
        run_basic_sumcheck_test(SpaceProver::new(test_polynomial()));
    }
}
