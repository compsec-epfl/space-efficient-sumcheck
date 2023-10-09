use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, Term};

pub trait Prover<F: Field, T: Term> {   
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(SparsePolynomial::<F, T>, SparsePolynomial::<F, T>)>;
    fn total_rounds(&self) -> usize;
}