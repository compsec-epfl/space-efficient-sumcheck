use ark_ff::Field;
use ark_std::vec::Vec;
use ark_poly::multivariate::{SparsePolynomial, Term};

use crate::sumcheck::Prover;

#[derive(Debug)]
pub struct MultilinearSumcheck<F: Field, T: Term> {
    pub prover_messages: Vec<(SparsePolynomial::<F, T>, SparsePolynomial::<F, T>)>,
    pub verifier_messages: Vec<F>,
}

impl<F: Field, T: Term> MultilinearSumcheck<F, T> {
    pub fn prove<P: Prover<F, T>>(mut prover: P) -> Self {
        let rounds = prover.total_rounds();
        let mut prover_messages = Vec::with_capacity(rounds);
        let mut verifier_messages = Vec::with_capacity(rounds);
 
        let mut verifier_message = None;
        while let Some((_0, _1)) = prover.next_message(verifier_message) {
            prover_messages.push((_0, _1));

            // simulate a challenge from the verifier
            verifier_message = Some(F::one());
            verifier_messages.push(F::one());
        }

        MultilinearSumcheck {
            prover_messages,
            verifier_messages,
        }
    }
}
