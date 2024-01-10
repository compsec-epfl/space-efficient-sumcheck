use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{evaluation_stream::EvaluationStream, prover::Prover};

// the state of the time prover in the protocol
pub struct TimeProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluations: Option<Vec<F>>,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>, // Keep this for now, case we can do some small optimizations of first round etc
    pub num_variables: usize,
}

impl<'a, F: Field> TimeProver<'a, F> {
    pub fn new(evaluation_stream: Box<&'a dyn EvaluationStream<F>>) -> Self {
        let claimed_sum = evaluation_stream.get_claimed_sum();
        let num_variables = evaluation_stream.get_num_variables();
        Self {
            claimed_sum,
            current_round: 0,
            evaluations: None,
            evaluation_stream,
            num_variables,
        }
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
    fn vsbw_evaluate(&self) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let bitmask: usize = 1 << self.num_free_variables() - 1;
        let evaluations_len = match &self.evaluations {
            Some(evaluations) => evaluations.len(),
            None => 2usize.pow(
                self.evaluation_stream
                    .get_num_variables()
                    .try_into()
                    .unwrap(),
            ),
        };
        for i in 0..evaluations_len {
            let is_set: bool = (i & bitmask) != 0;
            let point_evaluation = match &self.evaluations {
                Some(evaluations) => evaluations[i],
                None => self.evaluation_stream.get_evaluation_from_index(i),
            };
            match is_set {
                false => sum_0 += point_evaluation,
                true => sum_1 += point_evaluation,
            }
        }
        (sum_0, sum_1)
    }
    fn vsbw_reduce_evaluations(&mut self, verifier_message: F, verifier_message_hat: F) {
        let mut evaluations = match &self.evaluations {
            // all rounds after r=1 this table already exists
            Some(evaluations) => evaluations.clone(),
            // r=0 this function isn't called, r=1 we have to initialize this table and read in values from stream (2x size)
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
        let evaluations_len = match &self.evaluations {
            // either we iterate through only half of the table
            Some(evaluations) => evaluations.len() / 2,
            // or we just initialized and we need to iterate through all of it, reading from stream (2x size) as we go
            None => evaluations.len(),
        };
        let setbit: usize = 1 << self.num_free_variables(); // we use this to index the second half of the last round's evaluations e.g 001 AND 101
        for i0 in 0..evaluations_len {
            let i1 = i0 | setbit;
            let point_evaluation_i0 = match &self.evaluations {
                None => self.evaluation_stream.get_evaluation_from_index(i0),
                Some(evaluations) => evaluations[i0],
            };
            let point_evaluation_i1 = match &self.evaluations {
                None => self.evaluation_stream.get_evaluation_from_index(i1),
                Some(evaluations) => evaluations[i1],
            };
            evaluations[i0] =
                point_evaluation_i0 * verifier_message_hat + point_evaluation_i1 * verifier_message;
        }
        evaluations.truncate(evaluations_len);
        self.evaluations = Some(evaluations.clone());
    }
}

impl<'a, F: Field> Prover<F> for TimeProver<'a, F> {
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
        test_helpers::{
            run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
            BasicEvaluationStream, TestField,
        },
        TimeProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(TimeProver::new(Box::new(&evaluation_stream)));
        run_basic_sumcheck_test(TimeProver::new(Box::new(&evaluation_stream)));
    }
}
