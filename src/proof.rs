use ark_ff::Field;
use ark_std::{rand::Rng, vec::Vec};

use crate::provers::Prover;

#[derive(Debug)]
pub struct Sumcheck<F: Field> {
    pub prover_messages: Vec<(F, F)>,
    pub verifier_messages: Vec<F>,
    pub is_accepted: bool,
}

impl<F: Field> Sumcheck<F> {
    pub fn prove<P: Prover<F>, R: Rng>(prover: &mut P, rng: &mut R) -> Self {
        let mut prover_messages: Vec<(F, F)> = Vec::with_capacity(prover.total_rounds());
        let mut verifier_messages: Vec<F> = Vec::with_capacity(prover.total_rounds());
        let mut is_accepted = true;

        // run the protocol
        let mut verifier_message: Option<F> = None;
        while let Some(message) = prover.next_message(verifier_message) {
            // handle the current round
            let round_sum = message.0 + message.1;
            let is_round_accepted = match verifier_message {
                // if first round, compare to claimed_sum
                None => round_sum == prover.claimed_sum(),
                // else compute f(prev_verifier_msg) = prev_sum_0 - (prev_sum_0 - prev_sum_1) * prev_verifier_msg == round_sum
                Some(prev_verifier_message) => {
                    verifier_messages.push(prev_verifier_message); // do this when != None
                    let prev_prover_message = prover_messages.last().unwrap();
                    round_sum
                        == prev_prover_message.0
                            - (prev_prover_message.0 - prev_prover_message.1)
                                * prev_verifier_message
                }
            };

            // handle how to proceed
            prover_messages.push(message);
            if !is_round_accepted {
                is_accepted = false;
                break;
            }

            // TODO: (z-tech) want to implement capability for F::ONE and F::ZERO
            let mut random_message: F = F::rand(rng);
            while random_message == F::ONE || random_message == F::ZERO {
                random_message = F::rand(rng);
            }
            verifier_message = Some(random_message);
        }

        // done.
        Sumcheck {
            prover_messages,
            verifier_messages,
            is_accepted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sumcheck;
    use crate::provers::{
        test_helpers::{test_polynomial, BasicEvaluationStream, TestField},
        TimeProver,
    };

    #[test]
    fn basic() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        let mut prover = TimeProver::<TestField>::new(Box::new(&evaluation_stream));
        let rng = &mut ark_std::test_rng();
        let transcript = Sumcheck::<TestField>::prove(&mut prover, rng);
        assert_eq!(transcript.is_accepted, true);
    }
}
