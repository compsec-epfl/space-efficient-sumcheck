use ark_ff::Field;
use ark_std::vec::Vec;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Prover;
use crate::sumcheck::SumcheckMultivariatePolynomial;

// the state of the basic prover in the protocol
pub struct BasicProver<F: Field, P: SumcheckMultivariatePolynomial<F>> {
    pub mlp: P, // a polynomial that will be treated as multilinear
    pub mlp_claim: F, // the claimed evaluation of mpl
    pub random_challenges: Vec<F>,
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
            random_challenges: Vec::with_capacity(num_vars),
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
            let random_field_element: F = verifier_message.unwrap();
            self.random_challenges.push(random_field_element);
        }

        // fix variables with verifier challenges (if any)
        let mlp: P = self.mlp.fix_variables(&self.random_challenges);

        // don't forget to increment the round
        self.current_round += 1;
    
        // return a univariate polynomial evaluated over the current (smaller) hypercube
        return Some(mlp.to_univariate());
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
        PrimeField,
    };
    use ark_poly::{
        multivariate::{self, SparseTerm, Term},
        DenseMVPolynomial,
        Polynomial,
    };
    // use ark_std::{rand::Rng, test_rng};

    use pretty_assertions::assert_eq;

    #[derive(MontConfig)]
    #[modulus = "5"]
    #[generator = "2"]
    struct FrConfig;

    type Fp5 = Fp64<MontBackend<FrConfig, 1>>;
    type PolyFp5 = multivariate::SparsePolynomial::<Fp5, SparseTerm>;

    #[test]
    fn basic_prover() {
        // 2 *x_1^3 + x_1 * x_3 + x_2 * x_3
        let test_g = PolyFp5::from_coefficients_slice(
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
        let mut test_g_clone = test_g.clone();

        let mut test_prover = BasicProver::<Fp5, PolyFp5>::new(test_g);
        assert_eq!(test_prover.total_rounds(), 3, "should set the number of variables correctly");

        // FIRST ROUND
        // all variables free
        // 000 = 0
        // 001 = 0
        // 010 = 0
        // 100 = 2
        // 110 = 2
        // 101 = 3
        // 011 = 1
        // 111 = 4
        // sum = 12 mod 5 = 2
        let test_g0 = test_prover.next_message(None).unwrap();
        let test_claim_0: Fp5 = Fp5::from(12);
        let test_verifier_eval_1 = test_g0.evaluate(&Fp5::ZERO) + test_g0.evaluate(&Fp5::ONE);
        assert_eq!(test_claim_0, test_verifier_eval_1, "should form the correct first message");

        // SECOND ROUND
        // x1 fixed to 0
        // 000 = 0
        // 001 = 0
        // 010 = 0
        // 011 = 1
        // sum = 1 mod 5 = 1
        let test_g1 = test_prover.next_message(Some(Fp5::ZERO)).unwrap();
        let test_expected_poly_1 = PolyFp5::from_coefficients_slice(
            2,
            &[
                (
                    Fp5::from_bigint(1u32.into()).unwrap(),
                    multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
                ),
            ],
        );
        test_g_clone = test_g_clone.fix_variables(&[Fp5::ZERO]);
        assert_eq!(test_g_clone, test_expected_poly_1, "should reduce the polynomial with the fixed variable correctly");
        let test_claim_1: Fp5 = Fp5::from(1);
        let test_verifier_eval_1 = test_g1.evaluate(&Fp5::ZERO) + test_g1.evaluate(&Fp5::ONE);
        assert_eq!(test_claim_1, test_verifier_eval_1, "should form the correct second message");

        // LAST ROUND (only one free variable remaining)
        // x2 fixed to 1
        // 010 = 0
        // 011 = 1
        // sum = 1 mod 5 = 1
        let test_g2 = test_prover.next_message(Some(Fp5::ONE)).unwrap();
        let test_expected_poly_2 = PolyFp5::from_coefficients_slice(
            1,
            &[
                (
                    Fp5::from_bigint(1u32.into()).unwrap(),
                    multivariate::SparseTerm::new(vec![(0, 1)]),
                ),
            ],
        );
        test_g_clone = test_g_clone.fix_variables(&[Fp5::ONE]);
        assert_eq!(test_g_clone, test_expected_poly_2, "should reduce the polynomial with the fixed variable correctly");
        let test_claim_2: Fp5 = Fp5::from(1);
        let test_verifier_eval_2 = test_g2.evaluate(&Fp5::ZERO) + test_g2.evaluate(&Fp5::ONE);
        assert_eq!(test_claim_2, test_verifier_eval_2, "should form the correct third message");
    }
}