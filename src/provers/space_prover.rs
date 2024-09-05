use ark_ff::Field;
use ark_std::marker::PhantomData;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::Hypercube,
    lagrange_polynomial::LagrangePolynomial,
    prover::{Prover, ProverArgs},
};

pub struct SpaceProver<'a, F: Field, S: EvaluationStream<F>> {
    claimed_sum: F,
    current_round: usize,
    evaluation_stream: &'a S,
    num_variables: usize,
    verifier_messages: Vec<F>,
    verifier_message_hats: Vec<F>,
}

impl<'a, F: Field, S: EvaluationStream<F>> SpaceProver<'a, F, S> {
    fn cty_evaluate(&self) -> (F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0: F = F::ZERO;
        let mut sum_1: F = F::ZERO;

        // Create a bitmask for the number of free variables
        let bitmask: usize = 1 << (self.num_free_variables() - 1);

        // Iterate in two loops
        let num_vars_outer_loop = self.current_round;
        let num_vars_inner_loop = self.num_variables - num_vars_outer_loop;

        // Outer loop over a subset of variables
        for (index_outer, outer) in Hypercube::new(num_vars_outer_loop) {
            // Calculate the weight using Lagrange polynomial
            let lag_poly: F = LagrangePolynomial::lag_poly(
                self.verifier_messages.clone(),
                self.verifier_message_hats.clone(),
                outer,
            );

            if lag_poly == F::ZERO {
                // in this case the inner loop does nothing
                continue;
            }

            // Inner loop over all possible evaluations for the remaining variables
            for (index_inner, _inner) in Hypercube::new(num_vars_inner_loop) {
                // Calculate the evaluation index
                let evaluation_index = index_outer << num_vars_inner_loop | index_inner;

                // Check if the bit at the position specified by the bitmask is set
                let is_set: bool = (evaluation_index & bitmask) != 0;

                // Use match to accumulate the appropriate value based on whether the bit is set or not
                let inner_sum = self.evaluation_stream.get_evaluation(evaluation_index) * lag_poly;
                match is_set {
                    false => sum_0 += inner_sum,
                    true => sum_1 += inner_sum,
                }
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1)
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
}

impl<'a, F: Field, S: EvaluationStream<F>> Prover<'a, F, S> for SpaceProver<'a, F, S> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn generate_default_args(stream: &'a S) -> ProverArgs<'a, F, S> {
        ProverArgs {
            stream,
            stage_info: None,
            _phantom: PhantomData,
        }
    }

    fn new(prover_args: ProverArgs<'a, F, S>) -> Self {
        let claimed_sum = prover_args.stream.get_claimed_sum();
        let num_variables = prover_args.stream.get_num_variables();
        Self {
            claimed_sum,
            evaluation_stream: prover_args.stream,
            verifier_messages: Vec::<F>::with_capacity(num_variables), // TODO: could be halfed somehow
            verifier_message_hats: Vec::<F>::with_capacity(num_variables),
            current_round: 0,
            num_variables,
        }
    }

    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to verifier_messages
        if self.current_round != 0 {
            self.verifier_messages.push(verifier_message.unwrap());
            self.verifier_message_hats
                .push(F::ONE - verifier_message.unwrap());
        }

        // evaluate using cty
        let sums: (F, F) = self.cty_evaluate();

        // don't forget to increment the round
        self.current_round += 1;

        Some(sums)
    }

    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{
        test_helpers::{
            run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
            BasicEvaluationStream, TestField,
        },
        Prover, SpaceProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(SpaceProver::new(SpaceProver::generate_default_args(
            &evaluation_stream,
        )));
        run_basic_sumcheck_test(SpaceProver::new(SpaceProver::generate_default_args(
            &evaluation_stream,
        )));
    }
}
