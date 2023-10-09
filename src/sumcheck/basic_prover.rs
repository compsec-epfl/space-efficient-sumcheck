use ark_ff::Field;
use ark_std::vec::Vec;
use ark_poly::multivariate::{SparsePolynomial, Term};

use crate::sumcheck::Prover;

// the state of the basic prover in the protocol
pub struct BasicProver<F: Field, T: Term> {
    // there is no programmatic insurance this polynomial is actually multilinear
    // TODO(z-tech): we probably want this https://docs.rs/ark-poly/latest/ark_poly/evaluations/multivariate/multilinear/index.html
    pub multilinear_polynomial: SparsePolynomial::<F, T>,
    pub verifier_randomness: Vec<F>,
    pub current_round: usize,
    pub total_rounds: usize, // number of variables in the polynomial
}

impl<F: Field, T: Term> BasicProver<F, T> {
    /// Create a new basic prover.
    /// This will cause a copy of multilinear polynomial
    pub(crate) fn new(mlp: SparsePolynomial::<F, T>) -> Self {
        BasicProver {
            multilinear_polynomial: mlp.clone(),
            verifier_randomness: Vec::with_capacity(mlp.num_vars),
            current_round: 0,
            total_rounds: mlp. num_vars,
        }
    }
}

impl<F: Field, T: Term> Prover<F, T> for BasicProver<F, T> {
    /// a basic next-message function.
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(SparsePolynomial::<F, T>, SparsePolynomial::<F, T>)> {
        assert!(self.current_round >= self.total_rounds, "More rounds than needed.");
        self.current_round += 1;
        Some((self.multilinear_polynomial.clone(), self.multilinear_polynomial.clone()))
    }
    fn total_rounds(&self) -> usize {
        self.total_rounds
    }
}