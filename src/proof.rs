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
        // Initialize vectors to store prover and verifier messages
        let mut prover_messages: Vec<(F, F)> = Vec::with_capacity(prover.total_rounds());
        let mut verifier_messages: Vec<F> = Vec::with_capacity(prover.total_rounds());
        let mut is_accepted = true;

        // Run the protocol
        let mut verifier_message: Option<F> = None;
        while let Some(message) = prover.next_message(verifier_message) {
            let round_sum = message.0 + message.1;
            let is_round_accepted = match verifier_message {
                // If first round, compare to claimed_sum
                None => round_sum == prover.claimed_sum(),
                // Else compute f(prev_verifier_msg) = prev_sum_0 - (prev_sum_0 - prev_sum_1) * prev_verifier_msg == round_sum, store verifier message
                Some(prev_verifier_message) => {
                    verifier_messages.push(prev_verifier_message);
                    let prev_prover_message = prover_messages.last().unwrap();
                    round_sum
                        == prev_prover_message.0
                            - (prev_prover_message.0 - prev_prover_message.1)
                                * prev_verifier_message
                }
            };

            // Handle how to proceed
            prover_messages.push(message);
            if !is_round_accepted {
                is_accepted = false;
                break;
            }

            // Resample if randomness happens to be 1 or 0
            let mut random_message: F = F::rand(rng);
            while random_message == F::ONE || random_message == F::ZERO {
                random_message = F::rand(rng);
            }
            verifier_message = Some(random_message);
        }

        // Return a Sumcheck struct with the collected messages and acceptance status
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
        test_helpers::{BenchEvaluationStream, TestField},
        TimeProver, TradeoffProver,
    };

    #[test]
    fn algorithm_consistency() {
        // take an evaluation stream
        let evaluation_stream: BenchEvaluationStream<TestField> = BenchEvaluationStream::new(20);
        // initialize the provers
        let mut tradeoff_k4_prover =
            TradeoffProver::<TestField>::new(Box::new(&evaluation_stream), 4);
        let mut time_prover = TimeProver::<TestField>::new(Box::new(&evaluation_stream));
        // run them and get the transcript
        let tradeoff_prover_transcript = Sumcheck::<TestField>::prove(&mut tradeoff_k4_prover, &mut ark_std::test_rng());
        let time_prover_transcript = Sumcheck::<TestField>::prove(&mut time_prover, &mut ark_std::test_rng());
        // ensure the transcript is identical
        assert_eq!(
            time_prover_transcript.prover_messages,
            tradeoff_prover_transcript.prover_messages
        );
    }
}
