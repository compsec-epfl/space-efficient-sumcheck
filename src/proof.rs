use ark_ff::Field;
use ark_std::{marker::PhantomData, rand::Rng, vec::Vec};

use crate::provers::{evaluation_stream::EvaluationStream, Prover};

#[derive(Debug)]
pub struct Sumcheck<F: Field, S: EvaluationStream<F>> {
    pub prover_messages: Vec<(F, F)>,
    pub verifier_messages: Vec<F>,
    pub is_accepted: bool,
    _phantom: PhantomData<S>,
}

impl<'a, F: Field, S: EvaluationStream<F>> Sumcheck<F, S> {
    pub fn prove<P: Prover<'a, F, S>, R: Rng>(prover: &mut P, rng: &mut R) -> Self {
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
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::Sumcheck;
    use crate::{field_32::Field32, provers::{
        test_helpers::{BenchEvaluationStream, TestField},
        BlendyProver, Prover, ProverArgs, ProverArgsStageInfo, TimeProver,
    }};

    #[test]
    fn algorithm_consistency() {
        // take an evaluation stream
        let evaluation_stream: BenchEvaluationStream<TestField> = BenchEvaluationStream::new(20);
        // initialize the provers
        let mut blendy_k3_prover =
            BlendyProver::<TestField, BenchEvaluationStream<TestField>>::new(ProverArgs {
                stream: &evaluation_stream,
                stage_info: Some(ProverArgsStageInfo { num_stages: 3 }),
                _phantom: PhantomData,
            });
        let mut time_prover =
            TimeProver::<TestField, BenchEvaluationStream<TestField>>::new(TimeProver::<
                TestField,
                BenchEvaluationStream<TestField>,
            >::generate_default_args(
                &evaluation_stream
            ));
        // run them and get the transcript
        let blendy_prover_transcript =
            Sumcheck::<TestField, BenchEvaluationStream<TestField>>::prove(
                &mut blendy_k3_prover,
                &mut ark_std::test_rng(),
            );
        let time_prover_transcript = Sumcheck::<TestField, BenchEvaluationStream<TestField>>::prove(
            &mut time_prover,
            &mut ark_std::test_rng(),
        );
        // ensure the transcript is identical
        assert_eq!(
            time_prover_transcript.prover_messages,
            blendy_prover_transcript.prover_messages
        );
    }
}
