use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::Hypercube,
    interpolation::{
        lagrange_polynomial, BasicSequentialLagrangePolynomial, SequentialLagrangePolynomial,
    },
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
    pub verifier_message_hats: Vec<F>,
    pub sums: Vec<F>,
    pub stage_size: usize,
}

impl<'a, F: Field> TradeoffProver<'a, F> {
    pub fn new(evaluation_stream: Box<&'a dyn EvaluationStream<F>>, num_stages: usize) -> Self {
        assert!(
            evaluation_stream.get_num_variables() % num_stages == 0,
            "k must divide number of variables with no remainder"
        );
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
            verifier_message_hats: Vec::<F>::with_capacity(num_variables),
            sums: Vec::<F>::with_capacity(stage_size),
            stage_size,
        }
    }
    fn shift_and_one_fill(num: usize, shift_amount: usize) -> usize {
        (num << shift_amount) | (1 << shift_amount) - 1
    }
    fn compute_partial_sums(sums: &Vec<F>) -> Vec<F> {
        let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(sums.len());
        let mut running_sum = F::ZERO;
        for eval in sums {
            running_sum += eval;
            partial_sums.push(running_sum);
        }
        return partial_sums;
    }
    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }
    // sumUpdatef(r1):
    fn sum_update(&mut self) {
        // 0. declare these ranges for convenience
        let b1_num_vars: usize = self.current_stage() * self.stage_size; // := (s-1)l because we are zero-indexed
        let b2_num_vars: usize = self.stage_size; // := l
        let b3_num_vars: usize = self.num_variables - b1_num_vars - b2_num_vars; // := (k-s)l because we are zero-indexed
        // 1. Initialize SUM[b2] := 0 for each b2 ∈ {0,1}^l
        let mut sum: Vec<F> = vec![F::ZERO; Hypercube::pow2(b2_num_vars)];
        // 2. Initialize st := LagInit((s - l)l, r)
        let mut bslp: BasicSequentialLagrangePolynomial<F> = BasicSequentialLagrangePolynomial::new(
            self.verifier_messages.clone(),
            self.verifier_message_hats.clone(),
        );
        // 3. For each b1 ∈ {0,1}^(s-1)l
        for b1_index in 0..Hypercube::pow2(b1_num_vars) {
            // (a) Compute (LagPoly, st) := LagNext(st)
            let lag_poly = bslp.next();
            // (b) For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for b2_index in 0..Hypercube::pow2(b2_num_vars) {
                for b3_index in 0..Hypercube::pow2(b3_num_vars) {
                    let index = b1_index << (b2_num_vars + b3_num_vars)
                        | b2_index << b3_num_vars
                        | b3_index;
                    sum[b2_index] = sum[b2_index]
                        + lag_poly * self.evaluation_stream.get_evaluation_from_index(index);
                }
            }
        }
        self.sums = sum;
    }
    fn compute_round(&self, partial_sums: &Vec<F>) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let j_prime = self.current_round - (self.current_stage() * self.stage_size); // := j-(s-1)l
        let r_shift = self.current_stage() * self.stage_size;
        for (b2_start_index, b2_start) in Hypercube::new(j_prime + 1).enumerate() {
            let b2_start_index_0 = b2_start_index << (self.stage_size - j_prime - 1);
            let b2_start_index_1 =
                Self::shift_and_one_fill(b2_start_index, self.stage_size - j_prime - 1);
            let left_value: F = if b2_start_index_0 == 0 {
                F::ZERO
            } else {
                partial_sums[b2_start_index_0 - 1]
            };
            let right_value = partial_sums[b2_start_index_1];
            match *b2_start.last().unwrap() {
                false => {
                    let mut r2_start_0: Vec<F> =
                        self.verifier_messages[r_shift..(r_shift + j_prime)].to_vec();
                    r2_start_0.push(F::ZERO); // need to add ZERO to end
                    let mut r2_start_hat_0: Vec<F> =
                        self.verifier_message_hats[r_shift..(r_shift + j_prime)].to_vec();
                    r2_start_hat_0.push(F::ONE); // need to add ONE - ZERO to end
                    let lag_poly_0: F =
                        lagrange_polynomial(r2_start_0.clone(), r2_start_hat_0, b2_start.clone());
                    sum_0 += lag_poly_0 * (right_value - left_value);
                }
                true => {
                    let mut r2_start_1: Vec<F> =
                        self.verifier_messages[r_shift..(r_shift + j_prime)].to_vec();
                    r2_start_1.push(F::ONE); // need to add ONE to end
                    let mut r2_start_hat_1: Vec<F> =
                        self.verifier_message_hats[r_shift..(r_shift + j_prime)].to_vec();
                    r2_start_hat_1.push(F::ZERO); // need to add ONE - ONE to end
                    let lag_poly_1: F =
                        lagrange_polynomial(r2_start_1, r2_start_hat_1, b2_start.clone());
                    sum_1 += lag_poly_1 * (right_value - left_value);
                }
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
            self.verifier_message_hats
                .push(F::ONE - verifier_message.unwrap());
        }

        if self.current_round % self.stage_size == 0 {
            self.sum_update();
        }

        // compute the sum
        let sums: (F, F) = self.compute_round(&Self::compute_partial_sums(&self.sums));

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
            run_basic_sumcheck_test, test_polynomial, BasicEvaluationStream, TestField,
        },
        TradeoffProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 1));
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 3));
    }
}
