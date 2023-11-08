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
    pub fn prove<P: Prover<F>, R: Rng>(mut prover: P, rng: &mut R) -> Self {
        let mut prover_messages: Vec<(F, F)> = Vec::with_capacity(prover.total_rounds());
        let mut verifier_messages: Vec<F> = Vec::with_capacity(prover.total_rounds());
        let mut is_accepted = true;

        // run the protocol
        let mut verifier_message: Option<F> = None;
        while let Some(message) = prover.next_message(verifier_message) {
            // handle the current round
            let round_sum = message.0 + message.1;
            let is_round_accepted = match verifier_message {
                // if first round compare to claimed_sum
                None => round_sum == prover.claimed_sum(),
                // else compute f(prev_verifier_msg) = prev_sum_0 - (prev_sum_0 - prev_sum_1) * prev_verifier_msg == round_sum
                Some(prev_verifier_message) => {
                    verifier_messages.push(prev_verifier_message); // this can only be done when != None
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
            verifier_message = Some(F::rand(rng));
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
    use crate::provers::time_prover::TimeProver;
    use crate::provers::unit_test_helpers::{test_polynomial, TestField};

    #[test]
    fn basic() {
        let prover = TimeProver::<TestField>::new(test_polynomial());
        let rng = &mut ark_std::test_rng();
        let transcript = Sumcheck::<TestField>::prove(prover, rng);
        assert_eq!(transcript.is_accepted, true);
    }
}
