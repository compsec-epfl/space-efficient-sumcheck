use ark_ff::Field;
use ark_std::vec::Vec;
use ark_poly::univariate::SparsePolynomial;

use crate::sumcheck::Prover;

#[derive(Debug)]
pub struct Sumcheck<F: Field> {
    pub prover_messages: Vec<SparsePolynomial<F>>,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> Sumcheck<F> {
    pub fn prove<P: Prover<F>>(mut prover: P) -> Self {
        let rounds = prover.total_rounds();
        let mut prover_messages = Vec::with_capacity(rounds);
        let mut verifier_messages = Vec::with_capacity(rounds);
 
        let mut verifier_message = None;
        while let Some(message) = prover.next_message(verifier_message) {
            prover_messages.push(message);

            // simulate a challenge from the verifier
            verifier_message = Some(F::one());
            verifier_messages.push(F::one());
        }

        Sumcheck {
            prover_messages,
            verifier_messages,
        }
    }
}
