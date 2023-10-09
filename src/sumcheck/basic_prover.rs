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
        assert!(self.current_round >= self.num_vars, "More rounds than needed.");
        self.current_round += 1;
        Some(self.g.to_univariate())
    }
    fn total_rounds(&self) -> usize {
        self.num_vars
    }
}