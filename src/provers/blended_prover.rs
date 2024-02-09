use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::Hypercube,
    lagrange_polynomial::LagrangePolynomial,
    prover::{Prover, ProverArgs},
};

// the state of the Blended prover in the protocol
pub struct BlendedProver<'a, F: Field> {
    pub claimed_sum: F,
    pub current_round: usize,
    pub evaluation_stream: Box<&'a dyn EvaluationStream<F>>,
    pub num_stages: usize,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
    pub verifier_message_hats: Vec<F>,
    pub sums: Vec<F>,
    pub lag_polys: Vec<F>,
    pub stage_size: usize,
}

impl<'a, F: Field> BlendedProver<'a, F> {
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
    fn update_lag_polys(&mut self) {
        // Calculate j_prime as j-(s-1)l
        let j_prime = self.current_round - (self.current_stage() * self.stage_size);

        // We can't update in place, we must updated into a new vec and then replace the old one
        let mut updated: Vec<F> = vec![F::ONE; Hypercube::pow2(self.stage_size)];

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for b2_start_index in 0..Hypercube::pow2(j_prime + 1) {
            // calculate lag_poly from precomputed
            let lag_poly = match j_prime {
                0 => F::ONE,
                _ => {
                    let precomputed: F = *self.lag_polys.get(b2_start_index >> 1).unwrap();
                    match b2_start_index & 2 == 2 {
                        true => precomputed * *self.verifier_messages.last().unwrap(),
                        false => precomputed * *self.verifier_message_hats.last().unwrap(),
                    }
                }
            };
            updated[b2_start_index] = lag_poly;
        }
        self.lag_polys = updated;
    }
    fn compute_round(&mut self, partial_sums: &Vec<F>) -> (F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;

        // Calculate j_prime as j-(s-1)l
        let stage_start_index: usize = self.current_stage() * self.stage_size;
        let j_prime = self.current_round - stage_start_index;

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for (b2_start_index, b2_start) in Hypercube::new(j_prime + 1).enumerate() {
            // Calculate b2_start_index_0 and b2_start_index_1 for indexing partial_sums
            let shift_amount = if self.num_variables - stage_start_index < self.stage_size {
                // this is the oddly sized last stage when k doesn't divide num_vars
                self.num_variables - (self.current_stage() * self.stage_size) - j_prime - 1
            } else {
                self.stage_size - j_prime - 1
            };
            let b2_start_index_0: usize = b2_start_index << shift_amount;
            let b2_start_index_1: usize = Self::shift_and_one_fill(b2_start_index, shift_amount);

            // Calculate left_value and right_value based on partial_sums
            let left_value: F = match b2_start_index_0 {
                0 => F::ZERO,
                _ => partial_sums[b2_start_index_0 - 1],
            };
            let right_value = partial_sums[b2_start_index_1];
            let sum = right_value - left_value;

            // Match based on the last bit of b2_start
            match *b2_start.last().unwrap() {
                false => sum_0 += self.lag_polys[b2_start_index] * sum,
                true => sum_1 += self.lag_polys[b2_start_index] * sum,
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1)
    }
}

impl<'a, F: Field> Prover<'a, F> for BlendedProver<'a, F> {
    const DEFAULT_NUM_STAGES: usize = 2;
    fn new(prover_args: ProverArgs<'a, F>) -> Self {
        let claimed_sum = prover_args.stream.get_claimed_sum();
        let num_variables = prover_args.stream.get_num_variables();
        let stage_size: usize = num_variables / prover_args.num_stages;
        // return the BlendedProver instance
        Self {
            claimed_sum,
            current_round: 0,
            evaluation_stream: prover_args.stream,
            num_stages: prover_args.num_stages,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
            verifier_message_hats: Vec::<F>::with_capacity(num_variables),
            sums: Vec::<F>::with_capacity(stage_size),
            lag_polys: vec![F::ONE; Hypercube::pow2(stage_size)],
            stage_size,
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
        self.update_lag_polys();
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
        prover::ProverArgs,
        test_helpers::{
            run_basic_sumcheck_test, run_boolean_sumcheck_test, test_polynomial,
            BasicEvaluationStream, TestField,
        },
        BlendedProver, Prover,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_boolean_sumcheck_test(BlendedProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: BlendedProver::<TestField>::DEFAULT_NUM_STAGES,
        }));
        run_basic_sumcheck_test(BlendedProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: BlendedProver::<TestField>::DEFAULT_NUM_STAGES,
        }));
    }
}