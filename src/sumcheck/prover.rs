use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;

pub trait Prover<F: Field> {   
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>>;
    fn total_rounds(&self) -> usize;
}