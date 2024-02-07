use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream, hypercube::Hypercube, interpolation::LagrangePolynomial,
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
    fn compute_prefix_sums(sums: &Vec<F>) -> Vec<F> {
        sums.iter()
            .scan(F::ZERO, |sum, i| {
                *sum += i;
                Some(*sum)
            })
            .collect::<Vec<F>>()
    }
    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }
    fn sum_update(&mut self) {
        // 0. Declare ranges for convenience
        let b1_num_vars: usize = self.current_stage() * self.stage_size;
        let b2_num_vars: usize = if self.num_variables - b1_num_vars < self.stage_size {
            // this is the oddly sized last stage when k doesn't divide num_vars
            self.num_variables - b1_num_vars
        } else {
            self.stage_size
        };
        let b3_num_vars: usize = self.num_variables - b1_num_vars - b2_num_vars;

        // 1. Initialize SUM[b2] := 0 for each b2 ∈ {0,1}^l
        let mut sum: Vec<F> = vec![F::ZERO; Hypercube::pow2(b2_num_vars)];

        // 2. Initialize st := LagInit((s - l)l, r)
        let mut sequential_lag_poly: LagrangePolynomial<F> = LagrangePolynomial::new(
            self.verifier_messages.clone(),
            self.verifier_message_hats.clone(),
        );

        // 3. For each b1 ∈ {0,1}^(s-1)l
        for b1_index in 0..Hypercube::pow2(b1_num_vars) {
            // (a) Compute (LagPoly, st) := LagNext(st)
            let lag_poly = sequential_lag_poly.next().unwrap();

            // (b) For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for b2_index in 0..Hypercube::pow2(b2_num_vars) {
                for b3_index in 0..Hypercube::pow2(b3_num_vars) {
                    // Calculate the index for the current combination of b1, b2, and b3
                    let index = b1_index << (b2_num_vars + b3_num_vars)
                        | b2_index << b3_num_vars
                        | b3_index;

                    // Update SUM[b2]
                    sum[b2_index] =
                        sum[b2_index] + lag_poly * self.evaluation_stream.get_evaluation(index);
                }
            }
        }

        // Update the internal state with the new sums
        self.sums = sum;
    }
    fn compute_round(&self, partial_sums: &Vec<F>) -> (F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;

        // Calculate j_prime as j-(s-1)l
        let j_prime = self.current_round - (self.current_stage() * self.stage_size);

        // Calculate r_shift as s*l
        let r_shift = self.current_stage() * self.stage_size;

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for (b2_start_index, b2_start) in Hypercube::new(j_prime + 1).enumerate() {
            // Calculate b2_start_index_0 and b2_start_index_1 for indexing partial_sums
            let shift_amount = if self.num_variables - (self.current_stage() * self.stage_size)
                < self.stage_size
            {
                // this is the oddly sized last stage when k doesn't divide num_vars
                self.num_variables - (self.current_stage() * self.stage_size) - j_prime - 1
            } else {
                self.stage_size - j_prime - 1
            };
            let b2_start_index_0 = b2_start_index << shift_amount;
            let b2_start_index_1 = Self::shift_and_one_fill(b2_start_index, shift_amount);

            // Calculate the partial sum
            let left_value: F = match b2_start_index_0 {
                0 => F::ZERO,
                _ => partial_sums[b2_start_index_0 - 1],
            };
            let right_value: F = partial_sums[b2_start_index_1];
            let partial_sum: F = right_value - left_value;

            // calculate lag_poly
            let last_bit_b2: bool = *b2_start.last().unwrap();
            let lag_poly: F = LagrangePolynomial::lag_poly(
                self.verifier_messages[r_shift..(r_shift + j_prime)] // r2_start
                    .iter()
                    .copied()
                    .chain(std::iter::once(if last_bit_b2 { F::ONE } else { F::ZERO }))
                    .collect(),
                self.verifier_message_hats[r_shift..(r_shift + j_prime)] // r2_start_hat
                    .iter()
                    .copied()
                    .chain(std::iter::once(if last_bit_b2 { F::ZERO } else { F::ONE }))
                    .collect(),
                b2_start.clone(),
            );

            // update one of the sums based on last bit of b2_start
            match last_bit_b2 {
                false => sum_0 += lag_poly * partial_sum,
                true => sum_1 += lag_poly * partial_sum,
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1)
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
            // Store the verifier message and its complement
            self.verifier_messages.push(verifier_message.unwrap());
            self.verifier_message_hats
                .push(F::ONE - verifier_message.unwrap());
        }

        // If the current round is a multiple of the stage size, update the sums
        if self.current_round % self.stage_size == 0 {
            self.sum_update();
        }

        // Compute the sum based on partial sums
        let sums: (F, F) = self.compute_round(&Self::compute_prefix_sums(&self.sums));

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial sums
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
