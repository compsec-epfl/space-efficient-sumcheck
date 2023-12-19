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
    fn lag_init(l: usize, r: &Vec<F>) -> F {
        F::ONE
    }
    fn lag_next(st: F) -> (F, F)  {
        (F::ONE, F::ONE)
    }
    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }
    // sumUpdatef(r1):
    fn sum_update(&self) -> Vec<F> {
        // 0. declare these ranges for convenience
        let b1_num_vars: usize = self.current_stage() * self.stage_size; // := (s-1)l because we are zero-indexed
        let b2_num_vars: usize = self.stage_size; // := l
        let b3_num_vars: usize = self.num_variables - b1_num_vars - b2_num_vars; // := (k-s)l because we are zero-indexed
        // 1. Initialize SUM[b2] := 0 for each b2 ∈ {0,1}^l
        let mut sum: Vec<F> = vec![F::ZERO; Hypercube::<F>::pow2(self.stage_size)];
        // 2. Initialize st := LagInit((s - l)l, r)
        let mut st: F = Self::lag_init(b1_num_vars, &self.verifier_messages);
        let mut lag_poly = F::ONE;
        // 3. For each b1 ∈ {0,1}^(s-1)l
        for (b1_index, b1) in Hypercube::<F>::new(b1_num_vars).enumerate() {
            // (a) Compute (LagPoly, st) := LagNext(st)
            (lag_poly, st) = Self::lag_next(st);
            // For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for b2_index in 0..Hypercube::<F>::pow2(b2_num_vars) {
                for b3_index in 0..Hypercube::<F>::pow2(b3_num_vars) {
                    let index = b1_index << (b2_num_vars + b3_num_vars) | b2_index << b3_num_vars | b3_index;
                    sum[b2_index] = sum[b2_index] + lag_poly * self
                        .evaluation_stream
                        .get_evaluation_from_index(index);
                }
            }
        }
        return sum;
    }
    fn compute_round(&self, sums: Vec<F>) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let j_prime = self.current_round - (self.current_stage() * self.stage_size); // := j-(s-1)l
        for (b2_index, b2) in Hypercube::new(self.stage_size).enumerate() {
            let b2_start: &[F] = &b2[0..j_prime];
            let r2_start: &[F] = &self.verifier_messages[0..j_prime];
            let lag_poly: F = lagrange_polynomial(&b2_start, &r2_start).unwrap();
            // TODO: how do I know which sum this belongs to?
            sum_0 += lag_poly * sums[b2_index];
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
        let sums: (F, F) = self.compute_round(self.sum_update());

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
