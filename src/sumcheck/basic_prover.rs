use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Prover;
use crate::sumcheck::SumcheckMultivariatePolynomial;

// the state of the basic prover in the protocol
pub struct BasicProver<F: Field, P: SumcheckMultivariatePolynomial<F>> {
    pub multilinear_polynomial: P, // a polynomial that will be treated as multilinear
    pub claimed_evaluation: F,     // the claimed evaluation of the multilinear polynomial
    pub random_challenges: Vec<F>, // random challenges for the protocol
    pub current_round: usize,      // current round of the protocol
    pub num_variables: usize,      // number of variables in the multilinear polynomial
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> BasicProver<F, P> {
    // create new basic prover state
    pub fn new(multilinear_polynomial: P) -> Self {
        let claimed_evaluation = multilinear_polynomial.to_evaluations().into_iter().sum();
        let num_variables = multilinear_polynomial.num_vars();
        Self {
            multilinear_polynomial,
            claimed_evaluation,
            random_challenges: Vec::with_capacity(num_variables),
            current_round: 0,
            num_variables,
        }
    }
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> Prover<F> for BasicProver<F, P> {
    // Generates the next message for the verifier in the interactive protocol.
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to random_challenges
        if self.current_round != 0 {
            self.random_challenges.push(verifier_message.unwrap());
        }

        // Don't forget to increment the round
        self.current_round += 1;

        // Return a univariate polynomial evaluated over the current (smaller) hypercube
        let fixed_mlp = self
            .multilinear_polynomial
            .fix_variables(&self.random_challenges);
        Some(fixed_mlp.to_univariate())
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
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
    }
}

#[cfg(test)]
pub(crate) mod tests {
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
    pub struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;
    type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

    pub mod test_util {
        use super::*;
        pub fn tiny_test_polynomial() -> TestPolynomial {
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
    }

    #[test]
    fn basic_prover_init() {
        let prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
        assert_eq!(
            prover.total_rounds(),
            3,
            "should set the number of variables correctly"
        );
    }

    #[test]
    fn basic_prover_round_0() {
        // ZEROTH ROUND
        // all variables free
        // 000 = 0
        // 001 = 0
        // 010 = 13
        // 011 = 1
        // sum g0(0) = 14
        // 100 = 2
        // 110 = 0
        // 101 = 2
        // 111 = 7
        // sum g0(1) = 11
        let mut prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
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
    fn basic_prover_round_1() {
        // FIRST ROUND x0 fixed to 0
        // 101 = 2
        // 100 = 2
        // sum g1(0) = 4
        // 111 = 7
        // 110 = 0
        // sum g1(1) = 7
        let mut prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
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
    fn basic_prover_round_2() {
        // LAST ROUND x1 fixed to 1
        // 110 = 0
        // sum g(0) = 0
        // 111 = 7
        // sum g(1) = 7
        let mut prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
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
    fn basic_prover_outside_hypercube_round_1() {
        // FIRST ROUND x0 fixed to 3
        // 3,0,1 = 6
        // 3,0,0 = 6
        // sum g1(0) = 12
        // 3,1,1 = 38 = 0 mod 19
        // 3,1,0 = 31 = 12 mod 19
        // sum g1(1) = 12
        let mut prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
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
    fn basic_prover_outside_hypercube_round_2() {
        // LAST ROUND x1 fixed to 4
        // 3,4,0 = 108 = 11 mod 19
        // sum g(0) = 11
        // 3,4,1 = 138 = 1 mod 19
        // sum g(1) = 1
        let mut prover =
            BasicProver::<TestField, TestPolynomial>::new(test_util::tiny_test_polynomial());
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
