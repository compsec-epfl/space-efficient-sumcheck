use ark_ff::Field;

use crate::{
    multilinear_product::{TimeProductProver, TimeProductProverConfig},
    prover::Prover,
    streams::Stream,
};

impl<F: Field, S: Stream<F>> Prover<F> for TimeProductProver<F, S> {
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
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm};

    use crate::{
        multilinear_product::{ProductSumcheck, TimeProductProver},
        prover::{ProductProverConfig, Prover},
        streams::{MemoryStream, Stream},
        tests::{
            multilinear_product::sanity_test,
            multilinear_product::{BasicProductProver, ProductProverPolynomialConfig},
            polynomials::Polynomial,
            BenchStream, F19,
        },
    };
    #[test]
    fn sanity() {
        sanity_test::<F19, MemoryStream<F19>, TimeProductProver<F19, MemoryStream<F19>>>();
    }
    #[test]
    fn parity_with_basic_prover() {
        // take an evaluation stream
        const NUM_VARIABLES: usize = 16;
        let s: BenchStream<F19> = BenchStream::new(NUM_VARIABLES);
        let claim = s.claimed_sum;

        // prove over it using TimeProver
        let mut time_prover = TimeProductProver::<F19, BenchStream<F19>>::new(<TimeProductProver<
            F19,
            BenchStream<F19>,
        > as Prover<F19>>::ProverConfig::default(
            claim,
            NUM_VARIABLES,
            s.clone(),
            s.clone(),
        ));
        let time_prover_transcript = ProductSumcheck::<F19>::prove::<
            BenchStream<F19>,
            TimeProductProver<F19, BenchStream<F19>>,
        >(&mut time_prover, &mut ark_std::test_rng());

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
            BenchStream<F19>,
            BasicProductProver<F19>,
        >(&mut basic_prover, &mut ark_std::test_rng());

        // Assert they computed the same thing
        assert_eq!(
            basic_prover_transcript.prover_messages,
            time_prover_transcript.prover_messages
        );
    }
}
