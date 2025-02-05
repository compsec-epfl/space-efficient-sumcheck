use std::marker::PhantomData;

use ark_ff::Field;
use ark_std::vec::Vec;

use crate::{
    hypercube::Hypercube,
    interpolation::LagrangePolynomial,
    messages::VerifierMessages,
    multilinear_product::{Prover, ProverArgs, ProverArgsStageInfo},
    streams::EvaluationStream,
};

pub struct BlendyProductProver<'a, F: Field, S: EvaluationStream<F>> {
    claimed_sum: F,
    current_round: usize,
    stream_p: &'a S,
    stream_q: &'a S,
    num_stages: usize,
    num_variables: usize,
    verifier_messages: VerifierMessages<F>,
    x_table: Vec<F>,
    y_table: Vec<F>,
    j_prime_table: Vec<Vec<F>>,
    stage_size: usize,
}

impl<'a, F: Field, S: EvaluationStream<F>> BlendyProductProver<'a, F, S> {
    const DEFAULT_NUM_STAGES: usize = 2;

    fn compute_round(&self) -> (F, F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let mut sum_half = F::ZERO;

        let j_prime: usize;
        let t: usize;
        println!("num_variables: {:?}", self.num_variables);
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        println!("l: {:?}", l);
        let j = self.current_round + 1;
        println!("j: {:?}", j);
        let mut p = 0_usize;
        let s = (j).ilog2() as usize;
        println!("s: {:?}", s);
        println!("l - 1: {:?}", l - 1);
        if j <= l - 1 {
            if 2_usize.pow(s as u32) == j {
                p = 1;
            }
            j_prime = 2_usize.pow(s as u32);
            t = j_prime - 1;
        } else {
            if j % l == 0 {
                p = 1;
            }
            j_prime = l * (j / l);
            t = l - 1;
        }
        println!("j_prime: {:?}", j_prime);
        println!("t: {:?}", t);

        let r1: Vec<F> = self.verifier_messages.messages[0..j_prime - 1].to_vec();
        let mut r1_hat: Vec<F> = vec![F::ZERO; r1.len()];
        for i in 0..r1.len() {
            r1_hat[i] = F::ONE - r1[i];
        }
        let r2: Vec<F> = self.verifier_messages.messages[j_prime - 1..].to_vec();
        let mut r2_hat: Vec<F> = vec![F::ZERO; r2.len()];
        for i in 0..r2.len() {
            r2_hat[i] = F::ONE - r2[i];
        }

        for (b_prime_index, b_prime) in Hypercube::new(j - j_prime) {
            let lag_poly_1 =
                LagrangePolynomial::lag_poly(r2.clone(), r2_hat.clone(), b_prime.clone());
            for (b_prime_prime_index, b_prime_prime) in Hypercube::new(j - j_prime) {
                let lag_poly_2 =
                    LagrangePolynomial::lag_poly(r2.clone(), r2_hat.clone(), b_prime_prime.clone());
                println!("b_prime_prime: {:?}", b_prime_prime);
                println!("r1: {:?}", r1);
                println!("r2: {:?}", r2);
                println!("lag_poly_1: {:?}", lag_poly_1);
                println!("lag_poly_2: {:?}", lag_poly_2);
                for (v_index, _) in Hypercube::new((t as i64 - j as i64 + j_prime as i64) as usize)
                {
                    let b_prime_0_v =
                        v_index << (j - j_prime + 1) | 0 << (j - j_prime) | b_prime_index;
                    let b_prime_prime_0_v =
                        v_index << (j - j_prime + 1) | 0 << (j - j_prime) | b_prime_prime_index;
                    let b_prime_1_v =
                        v_index << (j - j_prime + 1) | 1 << (j - j_prime) | b_prime_index;
                    let b_prime_prime_1_v =
                        v_index << (j - j_prime + 1) | 1 << (j - j_prime) | b_prime_prime_index;
                    println!("b_prime_0_v {:?}", b_prime_0_v);
                    println!("b_prime_prime_0_v {:?}", b_prime_0_v);
                    println!(
                        "adding: {:?}",
                        self.j_prime_table[b_prime_0_v][b_prime_prime_0_v]
                    );
                    sum_0 += lag_poly_1
                        * lag_poly_2
                        * self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_1 += self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                    sum_half += self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_half += self.j_prime_table[b_prime_0_v][b_prime_prime_1_v];
                    sum_half += self.j_prime_table[b_prime_1_v][b_prime_prime_0_v];
                    sum_half += self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                }
                // sum_0 = sum_0 * lag_poly_1 * lag_poly_2;
                sum_1 = sum_1 * lag_poly_1 * lag_poly_2;
                sum_half = sum_half * lag_poly_1 * lag_poly_2;
            }
        }
        sum_half = sum_half / F::from(4_u32);

        // Return the accumulated sums
        (sum_0, sum_1, sum_half)
    }

    fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    fn compute_state(&mut self) {
        println!("####### compute state");
        let j_prime: usize;
        let t: usize;
        println!("num_variables: {:?}", self.num_variables);
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        println!("l: {:?}", l);
        let j = self.current_round + 1;
        println!("j: {:?}", j);
        let mut p = 0_usize;
        let s = (j).ilog2() as usize;
        println!("s: {:?}", s);
        println!("l - 1: {:?}", l - 1);
        if j <= l - 1 {
            if 2_usize.pow(s as u32) == j {
                p = 1;
            }
            j_prime = 2_usize.pow(s as u32);
            t = j_prime - 1;
        } else {
            if j % l == 0 {
                p = 1;
            }
            j_prime = l * (j / l);
            t = l - 1;
        }
        println!("j_prime: {:?}", j_prime);
        println!("t: {:?}", t);

        if p == 1 {
            self.j_prime_table = vec![
                vec![F::ZERO; Hypercube::stop_value(self.stage_size)];
                Hypercube::stop_value(self.stage_size)
            ];
            let b_num_vars = self.num_variables - j_prime - t;
            for (b_index, _) in Hypercube::new(b_num_vars) {
                println!("####### X update");

                for (b_prime_index, _) in Hypercube::new(t + 1) {
                    self.x_table[b_prime_index] = F::ZERO;
                    for (x_index, x) in Hypercube::new(j_prime - 1) {
                        let evaluation_point =
                            x_index << (t + 1 + b_num_vars) | b_prime_index << b_num_vars | b_index;
                        println!("point: {:?}", evaluation_point);
                        let lag_poly = LagrangePolynomial::lag_poly(
                            self.verifier_messages.messages[0..j_prime - 1].to_vec(),
                            self.verifier_messages.message_hats[0..j_prime - 1].to_vec(),
                            x,
                        );
                        println!(
                            "lag_poly: {:?}, eval: {:?}",
                            lag_poly,
                            self.stream_p.get_evaluation(evaluation_point)
                        );
                        self.x_table[b_prime_index] +=
                            lag_poly * self.stream_q.get_evaluation(evaluation_point);
                        println!("x_table: {:?}", self.x_table);
                    }
                }

                println!("####### Y update");

                for (b_prime_prime_index, _) in Hypercube::new(t + 1) {
                    self.y_table[b_prime_prime_index] = F::ZERO;
                    for (x_index, x) in Hypercube::new(j_prime - 1) {
                        let evaluation_point = x_index << (t + 1 + b_num_vars)
                            | b_prime_prime_index << b_num_vars
                            | b_index;
                        println!("b_prime_prime_index: {:?}", b_prime_prime_index);
                        println!("point: {:?}", evaluation_point);
                        let lag_poly = LagrangePolynomial::lag_poly(
                            self.verifier_messages.messages[0..j_prime - 1].to_vec(),
                            self.verifier_messages.message_hats[0..j_prime - 1].to_vec(),
                            x,
                        );
                        println!(
                            "lag_poly: {:?}, eval: {:?}",
                            lag_poly,
                            self.stream_p.get_evaluation(evaluation_point)
                        );
                        self.y_table[b_prime_prime_index] +=
                            lag_poly * self.stream_q.get_evaluation(evaluation_point);
                        println!("y_table: {:?}", self.y_table);
                    }
                }

                println!("####### M update");

                for (b_prime_index, _) in Hypercube::new(t + 1) {
                    for (b_prime_prime_index, _) in Hypercube::new(t + 1) {
                        // println!("adding to j_prime table: {:?}", (self.x_table[b_prime_index] * self.y_table[b_prime_prime_index]));
                        self.j_prime_table[b_prime_index][b_prime_prime_index] = self.j_prime_table
                            [b_prime_index][b_prime_prime_index]
                            + (self.x_table[b_prime_index] * self.y_table[b_prime_prime_index]);
                    }
                }
            }

            println!("########");
            println!("j_prime_table[0][0]: {:?}", self.j_prime_table[0][0]);
            println!("j_prime_table[0][1]: {:?}", self.j_prime_table[0][1]);
            println!("j_prime_table[1][0]: {:?}", self.j_prime_table[1][0]);
            println!("j_prime_table[1][1]: {:?}", self.j_prime_table[1][1]);
        }
    }
}

impl<'a, F: Field, S: EvaluationStream<F>> Prover<'a, F, S> for BlendyProductProver<'a, F, S> {
    fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    fn generate_default_args(
        stream_p: &'a S,
        stream_q: &'a S,
        claimed_sum: F,
    ) -> ProverArgs<'a, F, S> {
        ProverArgs {
            stream_p,
            stream_q,
            claimed_sum,
            stage_info: Some(ProverArgsStageInfo {
                num_stages: BlendyProductProver::<F, S>::DEFAULT_NUM_STAGES,
            }),
            _phantom: PhantomData,
        }
    }

    fn new(prover_args: ProverArgs<'a, F, S>) -> Self {
        let claimed_sum: F = prover_args.stream_p.get_claimed_sum();
        let num_variables: usize = prover_args.stream_q.get_num_variables();
        let num_stages: usize = prover_args.stage_info.unwrap().num_stages;
        let stage_size: usize = num_variables / num_stages;
        // return the BlendyProver instance
        Self {
            claimed_sum,
            current_round: 0,
            stream_p: prover_args.stream_p,
            stream_q: prover_args.stream_q,
            num_stages,
            num_variables,
            verifier_messages: VerifierMessages::new(&vec![]),
            x_table: vec![F::ZERO; Hypercube::stop_value(num_variables.div_ceil(2 * num_stages))],
            y_table: vec![F::ZERO; Hypercube::stop_value(num_variables.div_ceil(2 * num_stages))],
            j_prime_table: vec![
                vec![F::ZERO; Hypercube::stop_value(stage_size)];
                Hypercube::stop_value(stage_size)
            ],
            stage_size,
        }
    }

    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F, F)> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        if !self.is_initial_round() {
            self.verifier_messages
                .receive_message(verifier_message.unwrap());
        }

        self.compute_state();

        let sums: (F, F, F) = self.compute_round();

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

    use crate::{
        multilinear_product::{
            prover::{Prover, ProverArgs, ProverArgsStageInfo},
            BlendyProductProver,
        },
        tests::{four_variable_polynomial, sanity_test_4_variables, BasicEvaluationStream, F19},
    };

    #[test]
    fn sumcheck() {
        // create evaluation streams for a known polynomials
        let stream_p: BasicEvaluationStream<F19> =
            BasicEvaluationStream::new(four_variable_polynomial());
        let stream_q: BasicEvaluationStream<F19> =
            BasicEvaluationStream::new(four_variable_polynomial());

        // k=2 (DEFAULT)
        sanity_test_4_variables(BlendyProductProver::new(
            BlendyProductProver::generate_default_args(&stream_p, &stream_q, F19::from(18_u32)),
        ));
        // k=3
        sanity_test_4_variables(BlendyProductProver::new(ProverArgs {
            stream_p: &stream_p,
            stream_q: &stream_q,
            claimed_sum: F19::from(18_u32),
            stage_info: Some(ProverArgsStageInfo { num_stages: 3 }),
            _phantom: PhantomData,
        }));
    }
}
