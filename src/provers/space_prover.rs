use ark_ff::Field;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::Hypercube,
    lagrange_polynomial::LagrangePolynomial,
    prover::{Prover, ProverArgs},
};

pub struct SpaceProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluation_stream: &'a dyn EvaluationStream<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
    pub verifier_message_hats: Vec<F>,
}

impl<'a, F: Field> SpaceProver<'a, F> {
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
        for (index_outer, outer) in Hypercube::new(num_vars_outer_loop).enumerate() {
            // Calculate the weight using Lagrange polynomial
            let weight: F = LagrangePolynomial::lag_poly(
                self.verifier_messages.clone(),
                self.verifier_message_hats.clone(),
                outer,
            );

            // Inner loop over all possible evaluations for the remaining variables
            for index_inner in 0..2_usize.pow(num_vars_inner_loop as u32) {
                // Calculate the evaluation index
                let evaluation_index = index_outer << num_vars_inner_loop | index_inner;

                // Check if the bit at the position specified by the bitmask is set
                let is_set: bool = (evaluation_index & bitmask) != 0;

                // Use match to accumulate the appropriate value based on whether the bit is set or not
                match is_set {
                    false => {
                        sum_0 += self.evaluation_stream.get_evaluation(evaluation_index) * weight
                    }
                    true => {
                        sum_1 += self.evaluation_stream.get_evaluation(evaluation_index) * weight
                    }
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

impl<'a, F: Field> Prover<'a, F> for SpaceProver<'a, F> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn generate_default_args(stream: &'a impl EvaluationStream<F>) -> ProverArgs<'a, F> {
        ProverArgs {
            stream,
            stage_info: None,
        }
    }

    fn new(prover_args: ProverArgs<'a, F>) -> Self {
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
