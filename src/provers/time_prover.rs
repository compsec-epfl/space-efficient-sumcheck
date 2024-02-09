use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    prover::{Prover, ProverArgs},
};

// the state of the time prover in the protocol
pub struct TimeProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluations: Option<Vec<F>>,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>, // Keep this for now, case we can do some small optimizations of first round etc
    pub num_variables: usize,
}

impl<'a, F: Field> TimeProver<'a, F> {
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
    fn vsbw_evaluate(&self) -> (F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;

        // Calculate the bitmask for the number of free variables
        let bitmask: usize = 1 << (self.num_free_variables() - 1);

        // Determine the length of evaluations to iterate through
        let evaluations_len = match &self.evaluations {
            Some(evaluations) => evaluations.len(),
            None => 2usize.pow(self.evaluation_stream.get_num_variables() as u32),
        };

        // Iterate through evaluations
        for i in 0..evaluations_len {
            // Check if the bit at the position specified by the bitmask is set
            let is_set: bool = (i & bitmask) != 0;

            // Get the point evaluation for the current index
            let point_evaluation = match &self.evaluations {
                Some(evaluations) => evaluations[i],
                None => self.evaluation_stream.get_evaluation(i),
            };

            // Accumulate the value based on whether the bit is set or not
            match is_set {
                false => sum_0 += point_evaluation,
                true => sum_1 += point_evaluation,
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1)
    }
    fn vsbw_reduce_evaluations(&mut self, verifier_message: F, verifier_message_hat: F) {
        // Clone or initialize the evaluations vector
        let mut evaluations = match &self.evaluations {
            Some(evaluations) => evaluations.clone(),
            None => vec![
                F::ZERO;
                2usize.pow(
                    self.evaluation_stream
                        .get_num_variables()
                        .try_into()
                        .unwrap()
                ) / 2
            ],
        };

        // Determine the length of evaluations to iterate through
        let evaluations_len = match &self.evaluations {
            Some(evaluations) => evaluations.len() / 2,
            None => evaluations.len(),
        };

        // Calculate what bit needs to be set to index the second half of the last round's evaluations
        let setbit: usize = 1 << self.num_free_variables();

        // Iterate through pairs of evaluations
        for i0 in 0..evaluations_len {
            let i1 = i0 | setbit;

            // Get point evaluations for indices i0 and i1
            let point_evaluation_i0 = match &self.evaluations {
                None => self.evaluation_stream.get_evaluation(i0),
                Some(evaluations) => evaluations[i0],
            };
            let point_evaluation_i1 = match &self.evaluations {
                None => self.evaluation_stream.get_evaluation(i1),
                Some(evaluations) => evaluations[i1],
            };

            // Update the i0-th evaluation based on the reduction operation
            evaluations[i0] =
                point_evaluation_i0 * verifier_message_hat + point_evaluation_i1 * verifier_message;
        }

        // Truncate the evaluations vector to the correct length
        evaluations.truncate(evaluations_len);

        // Update the internal state with the new evaluations vector
        self.evaluations = Some(evaluations.clone());
    }
}

impl<'a, F: Field> Prover<'a, F> for TimeProver<'a, F> {
    const DEFAULT_NUM_STAGES: usize = 1;
    fn new(prover_args: ProverArgs<'a, F>) -> Self {
        let claimed_sum = prover_args.stream.get_claimed_sum();
        let num_variables = prover_args.stream.get_num_variables();
        Self {
            claimed_sum,
            current_round: 0,
            evaluations: None,
            evaluation_stream: prover_args.stream,
            num_variables,
        }
    }
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, reduce the evaluations table
        if self.current_round != 0 {
            // update the evaluations table by absorbing leftmost variable assigned to verifier_message
            self.vsbw_reduce_evaluations(
                verifier_message.unwrap(),
                F::ONE - verifier_message.unwrap(),
            )
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
    use crate::provers::{
        prover::ProverArgs,
        test_helpers::{
            run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
            BasicEvaluationStream, TestField,
        },
        Prover, TimeProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(TimeProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: TimeProver::<TestField>::DEFAULT_NUM_STAGES,
        }));
        run_basic_sumcheck_test(TimeProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: TimeProver::<TestField>::DEFAULT_NUM_STAGES,
        }));
    }
}
