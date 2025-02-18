use ark_ff::Field;

use crate::{
    multilinear_product::{TimeProductProver, TimeProductProverConfig},
    prover::Prover,
    streams::EvaluationStream,
};

impl<F: Field, S: EvaluationStream<F>> Prover<F> for TimeProductProver<F, S> {
    type ProverConfig = TimeProductProverConfig<F, S>;
    type ProverMessage = Option<(F, F, F)>;
    type VerifierMessage = Option<F>;

    fn claim(&self) -> F {
        self.claim
    }

    fn new(prover_config: Self::ProverConfig) -> Self {
        let num_variables = prover_config.num_variables;
        Self {
            claim: prover_config.claim,
            current_round: 0,
            evaluations_p: None,
            evaluations_q: None,
            stream_p: prover_config.stream_p,
            stream_q: prover_config.stream_q,
            num_variables,
            inverse_four: F::from(4_u32).inverse().unwrap(),
        }
    }

    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, reduce the evaluations table
        if self.current_round != 0 {
            // update the evaluations table by absorbing leftmost variable assigned to verifier_message
            self.vsbw_reduce_evaluations_p(
                verifier_message.unwrap(),
                F::ONE - verifier_message.unwrap(),
            );
            self.vsbw_reduce_evaluations_q(
                verifier_message.unwrap(),
                F::ONE - verifier_message.unwrap(),
            );
        }

        // evaluate using vsbw
        let sums = self.vsbw_evaluate();

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial
        return Some(sums);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        multilinear_product::TimeProductProver,
        tests::{multilinear_product::sanity_test, BasicEvaluationStream, F19},
    };
    #[test]
    fn sumcheck() {
        sanity_test::<
            F19,
            BasicEvaluationStream<F19>,
            TimeProductProver<F19, BasicEvaluationStream<F19>>,
        >();
    }
}
