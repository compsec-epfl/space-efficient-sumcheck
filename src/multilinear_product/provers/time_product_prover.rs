use ark_ff::Field;
use ark_std::{marker::PhantomData, vec::Vec};

use crate::{
    multilinear_product::{Prover, ProverArgs},
    streams::EvaluationStream,
};

pub struct TimeProductProver<'a, F: Field, S: EvaluationStream<F>> {
    claimed_sum: F,
    current_round: usize,
    evaluations_p: Option<Vec<F>>,
    evaluations_q: Option<Vec<F>>,
    stream_p: &'a S,
    stream_q: &'a S,
    num_variables: usize,
}

impl<'a, F: Field, S: EvaluationStream<F>> TimeProductProver<'a, F, S> {
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
    /*
     * Note in evaluate() there's an optimization for the first round where we read directly
     * from the streams (instead of the tables), which reduces max memory usage by 1/2
     */
    fn vsbw_evaluate(&self) -> (F, F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;

        // Calculate the bitmask for the number of free variables
        let bitmask: usize = 1 << (self.num_free_variables() - 1);

        // Determine the length of evaluations to iterate through
        let evaluations_len = match &self.evaluations_p {
            Some(evaluations) => evaluations.len(),
            None => 2usize.pow(self.stream_p.get_num_variables() as u32),
        };

        // Iterate through evaluations
        for i in 0..evaluations_len {
            // Check if the bit at the position specified by the bitmask is set
            let is_set: bool = (i & bitmask) != 0;

            // Get the point evaluation for the current index
            let product_of_points_evaluation = match (&self.evaluations_p, &self.evaluations_q) {
                (None, _) | (_, None) => {
                    self.stream_p.get_evaluation(i) * self.stream_q.get_evaluation(i)
                }
                (Some(evaluations_p), Some(evaluations_q)) => evaluations_p[i] * evaluations_q[i],
            };

            // Accumulate the value based on whether the bit is set or not
            match is_set {
                false => sum_0 += product_of_points_evaluation,
                true => sum_1 += product_of_points_evaluation,
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1, F::ZERO)
    }
    fn vsbw_reduce_evaluations_p(&mut self, verifier_message: F, verifier_message_hat: F) {
        // Clone or initialize the evaluations vector
        let mut evaluations = match &self.evaluations_p {
            Some(evaluations) => evaluations.clone(),
            None => {
                vec![F::ZERO; 2usize.pow(self.stream_p.get_num_variables().try_into().unwrap()) / 2]
            }
        };

        // Determine the length of evaluations to iterate through
        let evaluations_len = match &self.evaluations_p {
            Some(evaluations) => evaluations.len() / 2,
            None => evaluations.len(),
        };

        // Calculate what bit needs to be set to index the second half of the last round's evaluations
        let setbit: usize = 1 << self.num_free_variables();

        // Iterate through pairs of evaluations
        for i0 in 0..evaluations_len {
            let i1 = i0 | setbit;

            // Get point evaluations for indices i0 and i1
            let point_evaluation_i0 = match &self.evaluations_p {
                None => self.stream_p.get_evaluation(i0),
                Some(evaluations) => evaluations[i0],
            };
            let point_evaluation_i1 = match &self.evaluations_p {
                None => self.stream_p.get_evaluation(i1),
                Some(evaluations) => evaluations[i1],
            };
            // Update the i0-th evaluation based on the reduction operation
            evaluations[i0] =
                point_evaluation_i0 * verifier_message_hat + point_evaluation_i1 * verifier_message;
        }

        // Truncate the evaluations vector to the correct length
        evaluations.truncate(evaluations_len);

        // Update the internal state with the new evaluations vector
        self.evaluations_p = Some(evaluations.clone());
    }

    fn vsbw_reduce_evaluations_q(&mut self, verifier_message: F, verifier_message_hat: F) {
        // Clone or initialize the evaluations vector
        let mut evaluations = match &self.evaluations_q {
            Some(evaluations) => evaluations.clone(),
            None => {
                vec![F::ZERO; 2usize.pow(self.stream_q.get_num_variables().try_into().unwrap()) / 2]
            }
        };

        // Determine the length of evaluations to iterate through
        let evaluations_len = match &self.evaluations_q {
            Some(evaluations) => evaluations.len() / 2,
            None => evaluations.len(),
        };

        // Calculate what bit needs to be set to index the second half of the last round's evaluations
        let setbit: usize = 1 << self.num_free_variables();

        // Iterate through pairs of evaluations
        for i0 in 0..evaluations_len {
            let i1 = i0 | setbit;

            // Get point evaluations for indices i0 and i1
            let point_evaluation_i0 = match &self.evaluations_q {
                None => self.stream_q.get_evaluation(i0),
                Some(evaluations) => evaluations[i0],
            };
            let point_evaluation_i1 = match &self.evaluations_q {
                None => self.stream_q.get_evaluation(i1),
                Some(evaluations) => evaluations[i1],
            };
            // Update the i0-th evaluation based on the reduction operation
            evaluations[i0] =
                point_evaluation_i0 * verifier_message_hat + point_evaluation_i1 * verifier_message;
        }

        // Truncate the evaluations vector to the correct length
        evaluations.truncate(evaluations_len);

        // Update the internal state with the new evaluations vector
        self.evaluations_q = Some(evaluations.clone());
    }
}

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
