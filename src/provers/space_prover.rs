use ark_ff::Field;

use crate::provers::{
    evaluation_stream::EvaluationStream, hypercube::Hypercube, interpolation::lagrange_polynomial,
    Prover,
};

// the state of the space prover in the protocol
pub struct SpaceProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<'a, F: Field> SpaceProver<'a, F> {
    pub fn new(evaluation_stream: Box<&'a dyn EvaluationStream<F>>) -> Self {
        let claimed_sum = evaluation_stream.get_claimed_sum();
        let num_variables = evaluation_stream.get_num_variables();
        Self {
            claimed_sum,
            evaluation_stream,
            verifier_messages: Vec::<F>::with_capacity(num_variables), // TODO: could be halfed somehow
            current_round: 0,
            num_variables,
        }
    }
    // instance methods
    fn cty_evaluate(&self) -> (F, F) {
        let mut sum_0: F = F::ZERO;
        let mut sum_1: F = F::ZERO;
        let bitmask: usize = 1 << self.num_free_variables() - 1;
        // iterate in two loops
        let num_vars_outer_loop = self.current_round;
        let num_vars_inner_loop = self.num_variables - num_vars_outer_loop;
        for (index_outer, outer) in Hypercube::<F>::new(num_vars_outer_loop).enumerate() {
            let weight: F = lagrange_polynomial(&outer, &self.verifier_messages).unwrap();
            for index_inner in 0..2_usize.pow(num_vars_inner_loop as u32) {
                let evaluation_index = index_outer << num_vars_inner_loop | index_inner;
                let is_set: bool = (evaluation_index & bitmask) != 0;
                match is_set {
                    false => {
                        sum_0 += self
                            .evaluation_stream
                            .get_evaluation_from_index(evaluation_index)
                            * weight
                    }
                    true => {
                        sum_1 += self
                            .evaluation_stream
                            .get_evaluation_from_index(evaluation_index)
                            * weight
                    }
                }
            }
        }
        (sum_0, sum_1)
    }
    fn num_free_variables(&self) -> usize {
        self.num_variables - self.current_round
    }
}

impl<'a, F: Field> Prover<F> for SpaceProver<'a, F> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to verifier_messages
        if self.current_round != 0 {
            self.verifier_messages.push(verifier_message.unwrap());
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
        SpaceProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(SpaceProver::new(Box::new(&evaluation_stream)));
        run_basic_sumcheck_test(SpaceProver::new(Box::new(&evaluation_stream)));
    }
}
