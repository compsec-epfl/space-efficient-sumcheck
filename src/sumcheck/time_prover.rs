use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;
use ark_std::vec::Vec;

use crate::sumcheck::Prover;

// the state of the time prover in the protocol
pub struct TimeProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> TimeProver<F> {
    // class methods
    pub fn new(evaluations: Vec<F>, claimed_evaluation: F) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the TimeProver instance
        let num_variables: usize = (evaluations.len() as f64).log2() as usize;
        Self {
            claimed_evaluation,
            current_round: 0,
            evaluations,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
        }
    }
    // instance methods
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
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
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
        let (sum_0, sum_1) = self.vsbw_evaluate();

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

    use crate::sumcheck::polynomial::SumcheckMultivariatePolynomial;

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
        let test_evaluations = test_polynomial().to_evaluations();
        let prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        assert_eq!(
            prover.total_rounds(),
            3,
            "should set the number of variables correctly"
        );
    }

    #[test]
    fn round_0() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
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
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
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
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
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

    #[test]
    fn outside_hypercube_round_1() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::from(3))).unwrap(); // x0 fixed to 3
        assert_eq!(
            g_round_0.evaluate(&TestField::from(3)),
            g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ZERO),
            TestField::from(12),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            TestField::from(12),
            "g1 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn outside_hypercube_round_2() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover = TimeProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::from(3))).unwrap(); // x0 fixed to 3
        let g_round_2 = prover.next_message(Some(TestField::from(4))).unwrap(); // x1 fixed to 4
        assert_eq!(
            g_round_1.evaluate(&TestField::from(4)),
            g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ZERO),
            TestField::from(11),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ONE),
            TestField::from(1),
            "g2 should evaluate correctly for input 1"
        );
    }
}
