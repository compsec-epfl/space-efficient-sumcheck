use ark_ff::Field;

use crate::{
    hypercube::Hypercube,
    messages::VerifierMessages,
    multilinear_product::{BlendyProductProver, BlendyProductProverConfig},
    prover::Prover,
    streams::EvaluationStream,
};

impl<F: Field, S: EvaluationStream<F>> Prover<F> for BlendyProductProver<F, S> {
    type ProverConfig = BlendyProductProverConfig<F, S>;
    type ProverMessage = Option<(F, F, F)>;
    type VerifierMessage = Option<F>;

    fn claim(&self) -> F {
        self.claim
    }

    fn new(prover_config: Self::ProverConfig) -> Self {
        let num_variables: usize = prover_config.num_variables;
        let num_stages: usize = prover_config.num_stages;
        let stage_size: usize = num_variables / num_stages;
        // return the BlendyProver instance
        Self {
            claim: prover_config.claim,
            current_round: 0,
            stream_p: prover_config.stream_p,
            stream_q: prover_config.stream_q,
            num_stages,
            num_variables,
            verifier_messages: VerifierMessages::new(&vec![]),
            x_table: vec![F::ZERO; Hypercube::stop_value(num_variables.div_ceil(2 * num_stages))], // these correct?
            y_table: vec![F::ZERO; Hypercube::stop_value(num_variables.div_ceil(2 * num_stages))],
            j_prime_table: vec![
                vec![F::ZERO; Hypercube::stop_value(stage_size)]; // this correct?
                Hypercube::stop_value(stage_size)
            ],
            stage_size,
            inverse_four: F::from(4_u32).inverse().unwrap(),
        }
    }

    fn next_message(&mut self, verifier_message: Self::VerifierMessage) -> Self::ProverMessage {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        if !self.is_initial_round() {
            self.verifier_messages
                .receive_message(verifier_message.unwrap());
        }

        self.compute_state();

        let sums: (F, F, F) = self.compute_round();

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial sums
        Some(sums)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        multilinear_product::BlendyProductProver,
        tests::{multilinear_product::sanity_test, BasicEvaluationStream, F19},
    };
    #[test]
    fn sumcheck() {
        sanity_test::<
            F19,
            BasicEvaluationStream<F19>,
            BlendyProductProver<F19, BasicEvaluationStream<F19>>,
        >();

        // // create evaluation streams for a known polynomials
        // let stream_p: BasicEvaluationStream<F19> =
        //     BasicEvaluationStream::new(four_variable_polynomial_evaluations());
        // let stream_q: BasicEvaluationStream<F19> =
        //     BasicEvaluationStream::new(four_variable_polynomial_evaluations());

        // // k=2 (DEFAULT)
        // sanity_test_4_variables(BlendyProductProver::new(
        //     BlendyProductProver::generate_default_args(&stream_p, &stream_q, F19::from(18_u32)),
        // ));
        // // k=3
        // sanity_test_4_variables(BlendyProductProver::new(ProverArgs {
        //     stream_p: &stream_p,
        //     stream_q: &stream_q,
        //     claimed_sum: F19::from(18_u32),
        //     stage_info: Some(ProverArgsStageInfo { num_stages: 3 }),
        //     _phantom: PhantomData,
        // }));
    }
}
