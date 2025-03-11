use ark_ff::Field;
use ark_std::{rand::Rng, vec::Vec};

use crate::{prover::Prover, streams::Stream};

#[derive(Debug, PartialEq)]
pub struct ProductSumcheck<F: Field> {
    pub prover_messages: Vec<(F, F, F)>,
    pub verifier_messages: Vec<F>,
    pub is_accepted: bool,
}

// TODO: this can be moved to interpolation folder
fn evaluate_at<F: Field>(verifier_message: F, prover_message: (F, F, F)) -> F {
    // Hardcoded x-values:
    let zero = F::zero();
    let one = F::one();
    let half = F::from(2_u32).inverse().unwrap();

    // Compute denominators for the Lagrange basis polynomials
    let inv_denom_0 = ((zero - one) * (zero - half)).inverse().unwrap();
    let inv_denom_1 = ((one - zero) * (one - half)).inverse().unwrap();
    let inv_denom_2 = ((half - zero) * (half - one)).inverse().unwrap();

    // Compute the Lagrange basis polynomials evaluated at x
    let basis_p_0 = (verifier_message - one) * (verifier_message - half) * inv_denom_0;
    let basis_p_1 = (verifier_message - zero) * (verifier_message - half) * inv_denom_1;
    let basis_p_2 = (verifier_message - zero) * (verifier_message - one) * inv_denom_2;

    // Return the evaluation of the unique quadratic polynomial
    prover_message.0 * basis_p_0 + prover_message.1 * basis_p_1 + prover_message.2 * basis_p_2
}

impl<F: Field> ProductSumcheck<F> {
    pub fn prove<S, P>(prover: &mut P, rng: &mut impl Rng) -> Self
    where
        S: Stream<F>,
        P: Prover<F, VerifierMessage = Option<F>, ProverMessage = Option<(F, F, F)>>,
    {
        // Initialize vectors to store prover and verifier messages
        let mut prover_messages: Vec<(F, F, F)> = vec![];
        let mut verifier_messages: Vec<F> = vec![];
        let mut is_accepted = true;

        // Run the protocol
        let mut verifier_message: Option<F> = None;
        while let Some(message) = prover.next_message(verifier_message) {
            let round_sum = message.0 + message.1;
            let is_round_accepted = match verifier_message {
                // If first round, compare to claimed_sum
                None => round_sum == prover.claim(),
                Some(prev_verifier_message) => {
                    verifier_messages.push(prev_verifier_message);
                    let prev_prover_message = prover_messages.last().unwrap();
                    round_sum == evaluate_at(prev_verifier_message, *prev_prover_message)
                }
            };

            // Handle how to proceed
            prover_messages.push(message);
            if !is_round_accepted {
                is_accepted = false;
                break;
            }

            verifier_message = Some(F::rand(rng));
        }

        // Return a Sumcheck struct with the collected messages and acceptance status
        ProductSumcheck {
            prover_messages,
            verifier_messages,
            is_accepted,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        multilinear_product::{BlendyProductProver, TimeProductProver},
        tests::{multilinear_product::consistency_test, BenchStream, F64},
    };

    #[test]
    fn algorithm_consistency() {
        consistency_test::<F64, BenchStream<F64>, TimeProductProver<F64, BenchStream<F64>>>();
        consistency_test::<F64, BenchStream<F64>, BlendyProductProver<F64, BenchStream<F64>>>();
    }
}
