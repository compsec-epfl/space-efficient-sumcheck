use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Prover;
use crate::sumcheck::SumcheckMultivariatePolynomial;

// the state of the basic prover in the protocol
pub struct BasicProver<F: Field, P: SumcheckMultivariatePolynomial<F>> {
    pub mlp: P, // a polynomial that will be treated as multilinear
    pub mlp_claim: F, // the claimed evaluation of mpl
    pub verifier_randomness: Vec<F>,
    pub current_round: usize,
    pub num_vars: usize,
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> BasicProver<F, P> {
    // create new basic prover state
    pub fn new(mlp: P) -> Self {
        let mlp_claim = mlp.to_evaluations().into_iter().sum();
        let num_vars = mlp.num_vars();
        Self {
            mlp,
            mlp_claim,
            verifier_randomness: Vec::with_capacity(num_vars),
            current_round: 0,
            num_vars,
        }
    }
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> Prover<F> for BasicProver<F, P> {
    // a basic next-message function.
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        assert!(self.current_round <= self.total_rounds() - 1, "More rounds than needed."); // self.current_round is zero-indexed
        // first round only send univariate polynomial for verifier to check g0(0) + g0(1) = claim
        // all other rounds fix a variable with randomness from the verifier
        if self.current_round != 0 {
            // fix variables with verifier challenges (if any)
            let random_field_element: F = verifier_message.unwrap();
            self.verifier_randomness.push(random_field_element);
        }

        // don't forget to increment the round
        self.current_round += 1;
    
        // return a univariate polynomial evaluated over the current (smaller) hypercube
        let tmp_mlp = self.mlp.fix_variables(&self.verifier_randomness);
        return Some(tmp_mlp.to_univariate());
    }
    fn total_rounds(&self) -> usize {
        self.num_vars
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
        DenseMVPolynomial,
        Polynomial,
    };
    // use ark_std::{rand::Rng, test_rng};

    use pretty_assertions::assert_eq;

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;
    type TestPolynomial = multivariate::SparsePolynomial::<TestField, SparseTerm>;

    fn test_polynomial() -> TestPolynomial {
        // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2
        return TestPolynomial::from_coefficients_slice(
            3,
            &[
                (
                    TestField::from(4),
                    multivariate::SparseTerm::new(vec![(0, 1),(1, 1)]),
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
        )
    }

    #[test]
    fn basic_prover_init() {
        let test_prover = BasicProver::<TestField, TestPolynomial>::new(test_polynomial());
        assert_eq!(test_prover.total_rounds(), 3, "should set the number of variables correctly");
    }

    #[test]
    fn basic_prover_round_0() {
        // ZEROTH ROUND
        // all variables free
        // 000 = 0
        // 001 = 0
        // 010 = 13
        // 011 = 20
        // sum g(0) = 33 mod 19 = 14
        // 100 = 26
        // 110 = 19
        // 101 = 2
        // 111 = 2
        // sum g(1) = 49 mod 19 = 11
        let mut prover = BasicProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        assert_eq!(g_round_0.evaluate(&TestField::ZERO), TestField::from(14), "g0 should evaluate correctly for input 0");
        assert_eq!(g_round_0.evaluate(&TestField::ONE), TestField::from(11), "g0 should evaluate correctly for input 1");
    }

    #[test]
    fn basic_prover_round_1() {
        // FIRST ROUND x0 fixed to 1
        // 111 = 2
        // 101 = 2
        // sum g(0) = 4 mod 19 = 4
        // 100 = 26
        // 110 = 19
        // sum g(1) = 45 mod 19 = 7
        let mut prover = BasicProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        assert_eq!(g_round_0.evaluate(&TestField::ONE), g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE));
        assert_eq!(g_round_1.evaluate(&TestField::ZERO), TestField::from(4), "g1 should evaluate correctly for input 0");
        assert_eq!(g_round_1.evaluate(&TestField::ONE), TestField::from(7), "g1 should evaluate correctly for input 1");
    }

    #[test]
    fn basic_prover_round_2() {
        // LAST ROUND x1 fixed to 1
        // 110 = 19
        // sum g(0) = 19 mod 19 = 0 
        // 111 = 2
        // sum g(1) = 2 mod 19 = 2
        let mut prover = BasicProver::<TestField, TestPolynomial>::new(test_polynomial());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        let g_round_2 = prover.next_message(Some(TestField::ONE)).unwrap(); // x1 fixed to one
        assert_eq!(g_round_1.evaluate(&TestField::ONE), g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE));
        assert_eq!(g_round_2.evaluate(&TestField::ZERO), TestField::from(0), "g2 should evaluate correctly for input 0");
        assert_eq!(g_round_2.evaluate(&TestField::ONE), TestField::from(7), "g2 should evaluate correctly for input 1");
    }
}