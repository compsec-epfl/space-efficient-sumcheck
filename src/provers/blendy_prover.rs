use std::marker::PhantomData;

use ark_ff::Field;
use ark_std::vec::Vec;

use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::Hypercube,
    lagrange_polynomial::LagrangePolynomial,
    prover::{Prover, ProverArgs, ProverArgsStageInfo},
};

use super::verifier_messages::VerifierMessages;

pub struct BlendyProver<'a, F: Field, S: EvaluationStream<F>> {
    claimed_sum: F,
    current_round: usize,
    evaluation_stream: &'a S,
    num_stages: usize,
    num_variables: usize,
    // verifier_messages: Vec<F>,
    // verifier_message_hats: Vec<F>,
    vm: VerifierMessages<F>,
    sums: Vec<F>,
    lag_polys: Vec<F>,
    lag_polys_update: Vec<F>,
    stage_size: usize,
}

impl<'a, F: Field, S: EvaluationStream<F>> BlendyProver<'a, F, S> {
    const DEFAULT_NUM_STAGES: usize = 2;

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
        for (b2_start_index, _) in Hypercube::new(j_prime + 1) {
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
        // we reuse self.sums we just have to zero out on the first access SEE BELOW

        // 2. Initialize st := LagInit((s - l)l, r)
        let mut sequential_lag_poly: LagrangePolynomial<F> = LagrangePolynomial::new(
            self.vm.clone(),
            self.vm.messages.clone(),
            self.vm.message_hats.clone(),
        );

        // 3. For each b1 ∈ {0,1}^(s-1)l
        let len_sums: usize = self.sums.len();
        for (b1_index, _) in Hypercube::new(b1_num_vars) {
            // (a) Compute (LagPoly, st) := LagNext(st)
            let lag_poly = sequential_lag_poly.next().unwrap();

            // (b) For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for (b2_index, _) in Hypercube::new(b2_num_vars) {
                for (b3_index, _) in Hypercube::new(b3_num_vars) {
                    // Calculate the index for the current combination of b1, b2, and b3
                    let index = b1_index << (b2_num_vars + b3_num_vars)
                        | b2_index << b3_num_vars
                        | b3_index;

                    // Update SUM[b2]
                    self.sums[b2_index] =
                        match b1_index == 0 && b3_index == 0 && b2_index < len_sums {
                            // SEE HERE zero out the array on first access per update
                            true => lag_poly * self.evaluation_stream.get_evaluation(index),
                            false => {
                                self.sums[b2_index]
                                    + lag_poly * self.evaluation_stream.get_evaluation(index)
                            }
                        };
                }
            }
        }
    }
    fn update_lag_polys(&mut self) {
        // Calculate j_prime as j-(s-1)l
        let j_prime = self.current_round - (self.current_stage() * self.stage_size);

        // Iterate through b2_start indices using Hypercube::new(j_prime + 1)
        for (b2_start_index, _) in Hypercube::new(j_prime + 1) {
            // calculate lag_poly from precomputed
            let lag_poly = match j_prime {
                0 => F::ONE,
                _ => {
                    let precomputed: F = *self.lag_polys.get(b2_start_index >> 1).unwrap();
                    match b2_start_index & 2 == 2 {
                        true => precomputed * *self.vm.messages.last().unwrap(),
                        false => precomputed * *self.vm.message_hats.last().unwrap(),
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

impl<'a, F: Field, S: EvaluationStream<F>> Prover<'a, F, S> for BlendyProver<'a, F, S> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn generate_default_args(stream: &'a S) -> ProverArgs<'a, F, S> {
        ProverArgs {
            stream,
            stage_info: Some(ProverArgsStageInfo {
                num_stages: BlendyProver::<F, S>::DEFAULT_NUM_STAGES,
            }),
            _phantom: PhantomData,
        }
    }

    fn new(prover_args: ProverArgs<'a, F, S>) -> Self {
        let claimed_sum: F = prover_args.stream.get_claimed_sum();
        let num_variables: usize = prover_args.stream.get_num_variables();
        let num_stages: usize = prover_args.stage_info.unwrap().num_stages;
        let stage_size: usize = num_variables / num_stages;
        // return the BlendyProver instance
        Self {
            claimed_sum,
            current_round: 0,
            evaluation_stream: prover_args.stream,
            num_stages,
            num_variables,
            // verifier_messages: Vec::<F>::with_capacity(num_variables),
            // verifier_message_hats: Vec::<F>::with_capacity(num_variables),
            vm: VerifierMessages::new(),
            sums: vec![F::ZERO; Hypercube::stop_value(stage_size)],
            lag_polys: vec![F::ONE; Hypercube::stop_value(stage_size)],
            lag_polys_update: vec![F::ONE; Hypercube::stop_value(stage_size)],
            stage_size,
        }
    }

    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        if !self.is_initial_round() {
            self.vm.receive_message(verifier_message.unwrap());
            // Store the verifier message and its hat
            // self.verifier_messages.push(verifier_message.unwrap());
            // self.verifier_message_hats
            //     .push(F::ONE - verifier_message.unwrap());
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
    use std::marker::PhantomData;

    use crate::provers::{
        prover::{Prover, ProverArgs, ProverArgsStageInfo},
        test_helpers::{
            run_basic_sumcheck_test, test_polynomial, BasicEvaluationStream, TestField,
        },
        BlendyProver,
    };

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        // run_boolean_sumcheck_test(BlendyProver::new(BlendyProver::generate_default_args(
        //     &evaluation_stream,
        // )));
        // k=2
        run_basic_sumcheck_test(BlendyProver::new(BlendyProver::generate_default_args(
            &evaluation_stream,
        )));
        // k=1
        run_basic_sumcheck_test(BlendyProver::new(ProverArgs {
            stream: &evaluation_stream,
            stage_info: Some(ProverArgsStageInfo { num_stages: 1 }),
            _phantom: PhantomData,
        }));
    }
}
