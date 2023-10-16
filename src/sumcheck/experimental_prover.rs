use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;
use ark_std::vec::Vec;

use crate::sumcheck::Prover;
use crate::sumcheck::SumcheckMultivariatePolynomial;

pub struct ExperimentalProver<F: Field, P: SumcheckMultivariatePolynomial<F>> {
    pub multilinear_polynomial: P, // a polynomial that will be treated as multilinear
    pub claimed_evaluation: F,     // the claimed evaluation of the multilinear polynomial
    pub evaluations_per_input: Vec<F>, // evaluated values of the multilinear polynomial for each input
    pub range_sums: Vec<F>,            // range sums used in the computation
    pub random_challenges: Vec<F>,     // random challenges for the protocol
    pub current_round: usize,          // current round of the protocol
    pub num_variables: usize,          // number of variables in the multilinear polynomial
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> ExperimentalProver<F, P> {
    // create new time prover state
    pub fn new(multilinear_polynomial: P) -> Self {
        let num_variables = multilinear_polynomial.num_vars();
        // compute the input-output pairs
        let evaluations_per_input = multilinear_polynomial.to_evaluations();
        // compute the range sum lookup
        let mut running_sum = F::ZERO;
        let mut range_sums = Vec::<F>::with_capacity(num_variables);
        for point_eval in &evaluations_per_input {
            running_sum += point_eval;
            range_sums.push(running_sum);
        }
        // return ExperimentalProver instance
        Self {
            multilinear_polynomial,
            claimed_evaluation: range_sums[num_variables - 1],
            evaluations_per_input,
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
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> Prover<F> for ExperimentalProver<F, P> {
    // a next-message function using vsbw
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        assert!(
            self.current_round <= self.total_rounds() - 1,
            "More rounds than needed."
        );

        // If it's not the first round, add the verifier message to random_challenges
        if self.current_round != 0 {
            self.random_challenges.push(verifier_message.unwrap());
        }

        // Define the start and end of the range to sum over for g(0)
        let g_0_range_start_exclusive = [
            self.random_challenges.clone(),
            vec![F::ZERO],
            vec![F::ZERO; self.num_free_variables()],
        ]
        .concat();
        let g_0_range_end_inclusive = [
            self.random_challenges.clone(),
            vec![F::ZERO],
            vec![F::ONE; self.num_free_variables()],
        ]
        .concat();
        // Create range indices
        let sum_0_start_index_exclusive =
            ExperimentalProver::<F, P>::bits_to_index(&g_0_range_start_exclusive);
        let sum_0_end_index_inclusive =
            ExperimentalProver::<F, P>::bits_to_index(&g_0_range_end_inclusive);
        // Compute the sum of evaluations using the range lookup
        let mut g_0_evalutations_not_in_the_sum = F::ZERO;
        if sum_0_start_index_exclusive > 0 {
            g_0_evalutations_not_in_the_sum = self.range_sums[sum_0_start_index_exclusive - 1];
        }
        let sum_0 = self.range_sums[sum_0_end_index_inclusive] - g_0_evalutations_not_in_the_sum;

        // Define the start and end of the range to sum over for g(1)
        let g_1_range_start_exclusive = g_0_range_end_inclusive;
        let g_1_range_end_inclusive = [
            self.random_challenges.clone(),
            vec![F::ONE],
            vec![F::ONE; self.num_free_variables()],
        ]
        .concat();
        // Create range indices
        let sum_1_start_index_exclusive =
            ExperimentalProver::<F, P>::bits_to_index(&g_1_range_start_exclusive);
        let sum_1_end_index_inclusive =
            ExperimentalProver::<F, P>::bits_to_index(&g_1_range_end_inclusive);
        // Compute the sum of evaluations using the range lookup
        let g_1_evalutations_not_in_the_sum = self.range_sums[sum_1_start_index_exclusive];
        let sum_1 = self.range_sums[sum_1_end_index_inclusive] - g_1_evalutations_not_in_the_sum;

        // Form a polynomial s.t. g(0) = sum_0 and g(1) = sum_1
        let g: SparsePolynomial<F> =
            SparsePolynomial::<F>::from_coefficients_vec(vec![(0, sum_0), (1, -sum_0 + sum_1)]);

        // Increment the round counter
        self.current_round += 1;
        // Return the computed polynomial
        return Some(g);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
    fn num_free_variables(&self) -> usize {
        if self.num_variables == self.random_challenges.len() {
            return 0;
        }
        return self.num_variables - self.random_challenges.len() - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::{
        fields::Fp64,
        fields::{MontBackend, MontConfig},
    };
    use ark_poly::{
        multivariate::{self, SparseTerm, Term},
        DenseMVPolynomial, Polynomial,
    };

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;
    type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

    fn test_polynomial() -> TestPolynomial {
        // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2
        return TestPolynomial::from_coefficients_slice(
            3,
            &[
                (
                    TestField::from(4),
                    multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
                ),
                (
                    TestField::from(7),
                    multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
                ),
                (
                    TestField::from(2),
                    multivariate::SparseTerm::new(vec![(0, 1)]),
                ),
                (
                    TestField::from(13),
                    multivariate::SparseTerm::new(vec![(1, 1)]),
                ),
            ],
        );
    }

    #[test]
    fn init() {
        let prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        assert_eq!(
            prover.total_rounds(),
            3,
            "should set the number of variables correctly"
        );
    }

    #[test]
    fn round_0() {
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        assert_eq!(
            g_round_0.evaluate(&TestField::ZERO),
            TestField::from(14),
            "g0 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_0.evaluate(&TestField::ONE),
            TestField::from(11),
            "g0 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn round_1() {
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        assert_eq!(
            g_round_0.evaluate(&TestField::ONE),
            g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ZERO),
            TestField::from(4),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            TestField::from(7),
            "g1 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn round_2() {
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        let g_round_2 = prover.next_message(Some(TestField::ONE)).unwrap(); // x1 fixed to one
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ZERO),
            TestField::from(0),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ONE),
            TestField::from(7),
            "g2 should evaluate correctly for input 1"
        );
    }
}
