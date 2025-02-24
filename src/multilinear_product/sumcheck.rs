use ark_ff::Field;
use ark_std::{rand::Rng, vec::Vec};

use crate::{prover::Prover, streams::EvaluationStream};

#[derive(Debug, PartialEq)]
pub struct ProductSumcheck<F: Field> {
    pub prover_messages: Vec<(F, F, F)>,
    pub verifier_messages: Vec<F>,
    pub is_accepted: bool,
}

fn evaluate_at<F: Field>(verifier_message: F, prover_message: (F, F, F)) -> F {
    // Hardcoded x-values:
    let zero = F::zero();
    let one = F::one();
    let half = F::from(2_u32).inverse().unwrap();

    // Compute denominators for the Lagrange basis polynomials
    let inv_denom0 = ((zero - one) * (zero - half))
        .inverse()
        .expect("x0, x1, x2 must be distinct");
    let inv_denom1 = ((one - zero) * (one - half))
        .inverse()
        .expect("x0, x1, x2 must be distinct");
    let inv_denom2 = ((half - zero) * (half - one))
        .inverse()
        .expect("x0, x1, x2 must be distinct");

    // Compute the Lagrange basis polynomials evaluated at x
    let l0 = (verifier_message - one) * (verifier_message - half) * inv_denom0;
    let l1 = (verifier_message - zero) * (verifier_message - half) * inv_denom1;
    let l2 = (verifier_message - zero) * (verifier_message - one) * inv_denom2;

    // Return the evaluation of the unique quadratic polynomial
    prover_message.0 * l0 + prover_message.1 * l1 + prover_message.2 * l2
}

impl<F: Field> ProductSumcheck<F> {
    pub fn prove<S, P>(prover: &mut P, rng: &mut impl Rng) -> Self
    where
        S: EvaluationStream<F>,
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
    use super::ProductSumcheck;
    use crate::{
        multilinear_product::{BlendyProductProver, BlendyProductProverConfig, TimeProductProver},
        prover::{ProductProverConfig, Prover},
        tests::{BenchEvaluationStream, F19},
    };

    #[test]
    fn algorithm_consistency() {
        const NUM_VARIABLES: usize = 16;
        // take an evaluation stream
        let evaluation_stream: BenchEvaluationStream<F19> =
            BenchEvaluationStream::new(NUM_VARIABLES);
        let claim = F19::from(11);
        // initialize the provers
        let mut blendy_k2_prover = BlendyProductProver::<F19, BenchEvaluationStream<F19>>::new(
            BlendyProductProverConfig::new(
                claim,
                2,
                NUM_VARIABLES,
                evaluation_stream.clone(),
                evaluation_stream.clone(),
            ),
        );
        let blendy_prover_transcript = ProductSumcheck::<F19>::prove::<
            BenchEvaluationStream<F19>,
            BlendyProductProver<F19, BenchEvaluationStream<F19>>,
        >(&mut blendy_k2_prover, &mut ark_std::test_rng());

        let mut time_prover =
            TimeProductProver::<F19, BenchEvaluationStream<F19>>::new(<TimeProductProver<
                F19,
                BenchEvaluationStream<F19>,
            > as Prover<F19>>::ProverConfig::default(
                claim,
                NUM_VARIABLES,
                evaluation_stream.clone(),
                evaluation_stream,
            ));
        let time_prover_transcript = ProductSumcheck::<F19>::prove::<
            BenchEvaluationStream<F19>,
            TimeProductProver<F19, BenchEvaluationStream<F19>>,
        >(&mut time_prover, &mut ark_std::test_rng());
        // ensure the transcript is identical
        assert_eq!(time_prover_transcript.is_accepted, true);
        assert_eq!(time_prover_transcript, blendy_prover_transcript);
    }
}
