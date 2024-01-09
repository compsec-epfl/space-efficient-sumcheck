use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{evaluation_stream::EvaluationStream, prover::Prover};

// the state of the time prover in the protocol
pub struct TimeProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>, // Keep this for now, case we can do some small optimizations of first round etc
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
    pub verifier_message_hats: Vec<F>,
}

impl<'a, F: Field> TimeProver<'a, F> {
    pub fn new(evaluation_stream: Box<&'a dyn EvaluationStream<F>>) -> Self {
        let claimed_sum = evaluation_stream.get_claimed_sum();
        let num_variables = evaluation_stream.get_num_variables();
        let hypercube_len = 2usize.pow(num_variables.try_into().unwrap());
        let mut evaluations: Vec<F> = Vec::with_capacity(hypercube_len);
        for i in 0..hypercube_len {
            evaluations.push(evaluation_stream.get_evaluation_from_index(i));
        }
        Self {
            claimed_sum,
            current_round: 0,
            evaluations,
            evaluation_stream,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
            verifier_message_hats: Vec::<F>::with_capacity(num_variables),
        }
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
    fn vsbw_evaluate(&self) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let bitmask: usize = 1 << self.num_free_variables() - 1;
        for i in 0..self.evaluations.len() {
            let is_set: bool = (i & bitmask) != 0;
            match is_set {
                false => sum_0 += self.evaluations[i],
                true => sum_1 += self.evaluations[i],
            }
        }
        (sum_0, sum_1)
    }
    fn vsbw_reduce_evaluations(&mut self, verifier_message: F) {
        let half_size: usize = self.evaluations.len() / 2;
        let setbit: usize = 1 << self.num_free_variables(); // we use this to index the second half of the last round's evaluations e.g 001 AND 101
        for i0 in 0..half_size {
            let i1 = i0 | setbit;
            self.evaluations[i0] = self.evaluations[i0] * (F::ONE - verifier_message)
                + self.evaluations[i1] * verifier_message;
        }
        self.evaluations.truncate(half_size);
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
            self.vsbw_reduce_evaluations(verifier_message.unwrap())
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
