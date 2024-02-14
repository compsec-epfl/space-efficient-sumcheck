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
    pub lag_polys_update: Vec<F>,
    pub stage_size: usize,
}

impl<'a, F: Field> BlendedProver<'a, F> {
    fn shift_and_one_fill(num: usize, shift_amount: usize) -> usize {
        (num << shift_amount) | (1 << shift_amount) - 1
    }

    fn compute_round(&self, partial_sums: &[F]) -> (F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;

        // Calculate j_prime as j-(s-1)l
        let stage_start_index: usize = self.current_stage() * self.stage_size;
        let j_prime = self.current_round - stage_start_index;

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for b2_start_index in 0..Hypercube::stop_member_from_size(j_prime + 1) {
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
            match b2_start_index & 1 == 1 {
                false => sum_0 += self.lag_polys[b2_start_index] * sum,
                true => sum_1 += self.lag_polys[b2_start_index] * sum,
            }
        }

        // Return the accumulated sums
        (sum_0, sum_1)
    }

    fn current_stage(&self) -> usize {
        self.current_round / self.stage_size
    }

    fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    fn is_start_of_stage(&self) -> bool {
        self.current_round % self.stage_size == 0
    }

    fn is_single_staged(&self) -> bool {
        self.num_stages == 1
    }

    fn sum_update(&mut self) {
        if self.is_single_staged() {
            return;
        };
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
        // we reuse self.sums we just have to zero out on the first access
        let mut is_first_access: Vec<bool> =
            vec![true; Hypercube::stop_member_from_size(b2_num_vars)];

        // 2. Initialize st := LagInit((s - l)l, r)
        let mut sequential_lag_poly: LagrangePolynomial<F> = LagrangePolynomial::new(
            self.verifier_messages.clone(),
            self.verifier_message_hats.clone(),
        );

        // 3. For each b1 ∈ {0,1}^(s-1)l
        for b1_index in 0..Hypercube::stop_member_from_size(b1_num_vars) {
            // (a) Compute (LagPoly, st) := LagNext(st)
            let lag_poly = sequential_lag_poly.next().unwrap();

            // (b) For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for b2_index in 0..Hypercube::stop_member_from_size(b2_num_vars) {
                for b3_index in 0..Hypercube::stop_member_from_size(b3_num_vars) {
                    // Calculate the index for the current combination of b1, b2, and b3
                    let index = b1_index << (b2_num_vars + b3_num_vars)
                        | b2_index << b3_num_vars
                        | b3_index;

                    // Update SUM[b2]
                    self.sums[b2_index] = match is_first_access[b2_index] {
                        true => lag_poly * self.evaluation_stream.get_evaluation(index), // zero out the array on first access per update
                        false => {
                            self.sums[b2_index]
                                + lag_poly * self.evaluation_stream.get_evaluation(index)
                        }
                    };
                    is_first_access[b2_index] = false;
                }
            }
        }
    }
    fn update_lag_polys(&mut self) {
        // Calculate j_prime as j-(s-1)l
        let j_prime = self.current_round - (self.current_stage() * self.stage_size);

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for b2_start_index in 0..Hypercube::stop_member_from_size(j_prime + 1) {
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
            self.lag_polys_update[b2_start_index] = lag_poly;
        }
        std::mem::swap(&mut self.lag_polys, &mut self.lag_polys_update);
    }

    fn update_prefix_sums(&mut self) {
        self.sums = self
            .sums
            .clone()
            .into_iter()
            .enumerate()
            .scan(F::ZERO, |sum, (index, item)| {
                match self.is_single_staged() {
                    true => *sum += self.evaluation_stream.get_evaluation(index),
                    false => *sum += item,
                }
                Some(*sum)
            })
            .collect::<Vec<F>>();
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
            sums: vec![F::ZERO; Hypercube::stop_member_from_size(stage_size)],
            lag_polys: vec![F::ONE; Hypercube::stop_member_from_size(stage_size)],
            lag_polys_update: vec![F::ONE; Hypercube::stop_member_from_size(stage_size)],
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

        if !self.is_initial_round() {
            // Store the verifier message and its hat
            self.verifier_messages.push(verifier_message.unwrap());
            self.verifier_message_hats
                .push(F::ONE - verifier_message.unwrap());
        }

        // at start of stage do some stuff
        if self.is_start_of_stage() {
            self.sum_update();
            self.update_prefix_sums();
        }

        // update lag_polys based on previous round
        self.update_lag_polys();

        let sums: (F, F) = self.compute_round(&self.sums);

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
        // k=2
        run_basic_sumcheck_test(BlendedProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: BlendedProver::<TestField>::DEFAULT_NUM_STAGES,
        }));
        // k=1
        run_basic_sumcheck_test(BlendedProver::new(ProverArgs {
            stream: Box::new(&evaluation_stream),
            num_stages: 1,
        }));
    }
}
