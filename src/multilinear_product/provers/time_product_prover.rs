use ark_ff::Field;
use ark_std::{marker::PhantomData, vec::Vec};

use crate::{
    multilinear_product::{Prover, ProverArgs},
    streams::EvaluationStream,
};


impl<'a, F: Field, S: EvaluationStream<F>> Prover<'a, F, S> for TimeProductProver<'a, F, S> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn generate_default_args(
        stream_p: &'a S,
        stream_q: &'a S,
        claimed_sum: F,
    ) -> ProverArgs<'a, F, S> {
        ProverArgs {
            stream_p,
            stream_q,
            claimed_sum,
            stage_info: None,
            _phantom: PhantomData,
        }
    }

    fn new(prover_args: ProverArgs<'a, F, S>) -> Self {
        let claimed_sum = prover_args.stream_p.get_claimed_sum();
        let num_variables = prover_args.stream_p.get_num_variables();
        Self {
            claimed_sum,
            current_round: 0,
            evaluations_p: None,
            evaluations_q: None,
            stream_p: prover_args.stream_p,
            stream_q: prover_args.stream_q,
            num_variables,
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

    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        multilinear_product::{prover::Prover, TimeProductProver},
        tests::{four_variable_polynomial, sanity_test_4_variables, BasicEvaluationStream, F19},
    };

    #[test]
    fn sumcheck() {
        // create evaluation streams for a known polynomials
        let stream_p: BasicEvaluationStream<F19> =
            BasicEvaluationStream::new(four_variable_polynomial());
        let stream_q: BasicEvaluationStream<F19> =
            BasicEvaluationStream::new(four_variable_polynomial());

        // sanity check
        sanity_test_4_variables(TimeProductProver::new(
            TimeProductProver::generate_default_args(&stream_p, &stream_q, F19::from(18_u32)),
        ));
    }
}
