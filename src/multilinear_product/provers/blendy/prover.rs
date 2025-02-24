use ark_ff::Field;

use crate::{
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
            verifier_messages_round_comp: VerifierMessages::new(&vec![]),
            x_table: vec![],
            y_table: vec![],
            j_prime_table: vec![],
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
            // this holds everything
            self.verifier_messages
                .receive_message(verifier_message.unwrap());
            // this holds the randomness for between state computation r2
            self.verifier_messages_round_comp
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
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm};

    use crate::{
        multilinear_product::BlendyProductProver,
        prover::{ProductProverConfig, Prover},
        streams::EvaluationStream,
        tests::{
            multilinear_product::{sanity_test, BasicProductProver, ProductProverPolynomialConfig},
            polynomials::Polynomial,
            BasicEvaluationStream, BenchEvaluationStream, F19,
        },
        ProductSumcheck,
    };
    #[test]
    fn sumcheck() {
        sanity_test::<
            F19,
            BasicEvaluationStream<F19>,
            BlendyProductProver<F19, BasicEvaluationStream<F19>>,
        >();
    }

    #[test]
    fn parity_with_basic_prover() {
        // take an evaluation stream
        const NUM_VARIABLES: usize = 16;
        let s: BenchEvaluationStream<F19> = BenchEvaluationStream::new(NUM_VARIABLES);
        let claim = s.claimed_sum;

        // prove over it using BlendyProver
        let mut blendy_prover =
            BlendyProductProver::<F19, BenchEvaluationStream<F19>>::new(<BlendyProductProver<
                F19,
                BenchEvaluationStream<F19>,
            > as Prover<F19>>::ProverConfig::default(
                claim,
                NUM_VARIABLES,
                s.clone(),
                s.clone(),
            ));
        let blendy_prover_transcript = ProductSumcheck::<F19>::prove::<
            BenchEvaluationStream<F19>,
            BlendyProductProver<F19, BenchEvaluationStream<F19>>,
        >(&mut blendy_prover, &mut ark_std::test_rng());

        // Prove over it using BasicProver
        let p_evaluations: Vec<F19> = (0..1 << NUM_VARIABLES).map(|i| s.evaluation(i)).collect();
        let p: SparsePolynomial<F19, SparseTerm> =
            <SparsePolynomial<F19, SparseTerm> as Polynomial<F19>>::from_hypercube_evaluations(
                p_evaluations.clone(),
            );
        let mut basic_prover = BasicProductProver::<F19>::new(
            <BasicProductProver<F19> as Prover<F19>>::ProverConfig::default(
                claim,
                NUM_VARIABLES,
                p.clone(),
                p,
            ),
        );
        let basic_prover_transcript = ProductSumcheck::<F19>::prove::<
            BenchEvaluationStream<F19>,
            BasicProductProver<F19>,
        >(&mut basic_prover, &mut ark_std::test_rng());

        // Assert they computed the same thing
        assert_eq!(
            basic_prover_transcript.prover_messages,
            blendy_prover_transcript.prover_messages
        );
    }
}
