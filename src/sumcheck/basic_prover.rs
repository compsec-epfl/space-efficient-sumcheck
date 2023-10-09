use ark_ff::Field;
use ark_std::vec::Vec;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Prover;

use super::SumcheckPolynomial;

// the state of the basic prover in the protocol
pub struct BasicProver<F: Field, P: SumcheckPolynomial<F>> {
    pub g: P, // instance of the polynomial used for the protocol
    pub claimed_value: F,
    pub verifier_randomness: Vec<F>,
    pub current_round: usize,
    pub num_vars: usize,
}

impl<F: Field, P: SumcheckPolynomial<F>> BasicProver<F, P> {
    /// Create a new [`Prover`] state with the polynomial $g$.
    pub fn new(g: P) -> Self {
        let claimed_value = g.to_evaluations().into_iter().sum();
        let num_vars = g.num_vars();
        Self {
            g,
            claimed_value,
            verifier_randomness: Vec::with_capacity(num_vars),
            current_round: 0,
            num_vars,
        }
    }
}

impl<F: Field, P: SumcheckPolynomial<F>> Prover<F> for BasicProver<F, P> {
    /// a basic next-message function.
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        assert!(self.current_round <= self.num_vars, "More rounds than needed.");
        if self.current_round != 0 {
            let r = verifier_message.unwrap();
            self.verifier_randomness.push(r);
            self.g.fix_variables(&[r]);
        }
        self.current_round += 1;
        return Some(self.g.to_univariate());
    }
    fn total_rounds(&self) -> usize {
        self.num_vars
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::{
        fields::Fp64,
        fields::{MontBackend, MontConfig},
        Field, One, PrimeField,
    };
    use ark_poly::{
        multivariate::{self, SparseTerm, Term},
        DenseMVPolynomial,
        Polynomial,
    };
    use ark_std::{rand::Rng, test_rng};
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(MontConfig)]
    #[modulus = "5"]
    #[generator = "2"]
    struct FrConfig;

    type Fp5 = Fp64<MontBackend<FrConfig, 1>>;
    type TestPoly = multivariate::SparsePolynomial::<Fp5, SparseTerm>;

    #[test]
    fn basic_tests_next_message() {
        // 2 *x_1^3 + x_1 * x_3 + x_2 * x_3
        let g = TestPoly::from_coefficients_slice(
            3,
            &[
                (
                    Fp5::from_bigint(2u32.into()).unwrap(),
                    multivariate::SparseTerm::new(vec![(0, 3)]),
                ),
                (
                    Fp5::from_bigint(1u32.into()).unwrap(),
                    multivariate::SparseTerm::new(vec![(0, 1), (2, 1)]),
                ),
                (
                    Fp5::from_bigint(1u32.into()).unwrap(),
                    multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
                ),
            ],
        );
        // did it set the number of vars correctly?
        let mut p = BasicProver::<Fp5, TestPoly>::new(g);
        assert_eq!(p.total_rounds(), 3); // there are three variables

        // did it form the correct first message? (a univariate polynomial that sums to 12)
        let g0 = p.next_message(None).unwrap();
        let expected_sum = Fp5::from(12);
        let verifier_computed = g0.evaluate(&Fp5::from(0)) + g0.evaluate(&Fp5::from(1));
        assert_eq!(verifier_computed, expected_sum);
    }
}