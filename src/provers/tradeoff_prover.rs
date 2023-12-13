use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream, hypercube::Hypercube, interpolation::lagrange_polynomial,
    Prover,
};

// the state of the tradeoff prover in the protocol
pub struct TradeoffProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>,
    pub num_stages: usize,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
    pub stage_size: usize,
}

impl<'a, F: Field> TradeoffProver<'a, F> {
    pub fn new(evaluation_stream: Box<&'a dyn EvaluationStream<F>>, num_stages: usize) -> Self {
        let claimed_sum = evaluation_stream.get_claimed_sum();
        let num_variables = evaluation_stream.get_num_variables();
        let stage_size: usize = num_variables / num_stages;
        // return the TradeoffProver instance
        Self {
            claimed_sum,
            current_round: 0,
            evaluation_stream,
            num_stages,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
            stage_size,
        }
    }
    fn compute_partial_sums(precomputed: Vec<F>) -> Vec<F> {
        let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(precomputed.len());
        let mut running_sum = F::ZERO;
        for eval in &precomputed {
            running_sum += eval;
            partial_sums.push(running_sum);
        }
        return partial_sums;
    }
    fn shift_and_one_fill(num: usize, shift_amount: usize) -> usize {
        (num << shift_amount) | (1 << shift_amount) - 1
    }
    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }
    fn precompute_stage_evaluations(&self) -> Vec<F> {
        // define the ranges like so
        let num_vars_b1: usize = self.current_stage() * self.stage_size;
        let num_vars_b2: usize = self.stage_size;
        let num_vars_b3: usize = (self.num_stages - self.current_stage() - 1) * self.stage_size;
        // precompute the evaluations
        let mut precomputed: Vec<F> = vec![F::ZERO; 2_usize.pow(num_vars_b2 as u32)];
        for (index_b1, b1) in Hypercube::<F>::new(num_vars_b1).enumerate() {
            let weight: F = lagrange_polynomial(&b1, &self.verifier_messages[0..b1.len()]).unwrap();
            for index_b2 in 0..2_usize.pow(num_vars_b2 as u32) {
                for index_b3 in 0..2_usize.pow(num_vars_b3 as u32) {
                    let evaluations_index =
                        index_b1 << num_vars_b2 + num_vars_b3 | index_b2 << num_vars_b3 | index_b3;
                    precomputed[index_b2] = precomputed[index_b2]
                        + weight
                            * self
                                .evaluation_stream
                                .get_evaluation_from_index(evaluations_index);
                }
            }
        }
        return precomputed;
    }
    fn evaluate(&self, partial_sums: Vec<F>) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let num_vars_b1 = self.current_stage() * self.stage_size;
        let num_vars_b2_prime = self.current_round - num_vars_b1;
        let inner_index_shift = self.stage_size - num_vars_b2_prime - 1;
        for (index_b2_prime, b2_prime) in Hypercube::new(num_vars_b2_prime).enumerate() {
            let weight: F =
                lagrange_polynomial(&b2_prime, &self.verifier_messages[0..b2_prime.len()]).unwrap();
            if weight != F::ZERO {
                let start_0: usize = (index_b2_prime << 1) << inner_index_shift;
                let end_0: usize =
                    TradeoffProver::<F>::shift_and_one_fill(index_b2_prime << 1, inner_index_shift);
                let start_1: usize = (TradeoffProver::<F>::shift_and_one_fill(index_b2_prime, 1)
                    << inner_index_shift)
                    - 1;
                let end_1: usize = TradeoffProver::<F>::shift_and_one_fill(
                    TradeoffProver::<F>::shift_and_one_fill(index_b2_prime, 1),
                    inner_index_shift,
                );
                sum_0 += if start_0 == 0 {
                    partial_sums[end_0] * weight
                } else {
                    (partial_sums[end_0] - partial_sums[start_0 - 1]) * weight
                };
                sum_1 += (partial_sums[end_1] - partial_sums[start_1]) * weight;
            }
        }
        return (sum_0, sum_1);
    }
}

impl<'a, F: Field> Prover<F> for TradeoffProver<'a, F> {
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
            // store the verifier message
            self.verifier_messages.push(verifier_message.unwrap());
        }

        // compute the sum
        let sums: (F, F) = self.evaluate(TradeoffProver::compute_partial_sums(
            self.precompute_stage_evaluations(),
        ));

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
        TradeoffProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 1));
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 1));
        run_boolean_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 3));
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 3));
    }
}
