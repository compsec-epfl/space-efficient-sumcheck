use ark_ff::Field;
use ark_std::vec::Vec;

use crate::{
    hypercube::Hypercube, interpolation::LagrangePolynomial, messages::VerifierMessages,
    streams::EvaluationStream,
};

pub struct BlendyProductProver<F: Field, S: EvaluationStream<F>> {
    pub claim: F,
    pub current_round: usize,
    pub stream_p: S,
    pub stream_q: S,
    pub num_stages: usize,
    pub num_variables: usize,
    pub verifier_messages: VerifierMessages<F>,
    pub x_table: Vec<F>,
    pub y_table: Vec<F>,
    pub j_prime_table: Vec<Vec<F>>,
    pub stage_size: usize,
    pub inverse_four: F,
}

impl<F: Field, S: EvaluationStream<F>> BlendyProductProver<F, S> {
    pub fn total_rounds(&self) -> usize {
        self.num_variables
    }

    pub fn compute_round(&self) -> (F, F, F) {
        // Initialize accumulators for sum_0 and sum_1
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let mut sum_half = F::ZERO;

        let j_prime: usize;
        let t: usize;
        // println!("num_variables: {:?}", self.num_variables);
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        // println!("l: {:?}", l);
        let j = self.current_round + 1;
        // println!("j: {:?}", j);
        let p = 0_usize;
        let s = (j).ilog2() as usize;
        // println!("s: {:?}", s);
        // println!("l - 1: {:?}", l - 1);
        if j <= l - 1 {
            j_prime = 2_usize.pow(s as u32);
            t = j_prime - 1;
        } else {
            j_prime = l * (j / l);
            t = l - 1;
        }

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
                    sum_0 += self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_1 += self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                    sum_half += self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_half += self.j_prime_table[b_prime_0_v][b_prime_prime_1_v];
                    sum_half += self.j_prime_table[b_prime_1_v][b_prime_prime_0_v];
                    sum_half += self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                }
                sum_0 = sum_0 * lag_poly_1 * lag_poly_2;
                sum_1 = sum_1 * lag_poly_1 * lag_poly_2;
                sum_half = sum_half * lag_poly_1 * lag_poly_2;
            }
        }
        sum_half = sum_half * self.inverse_four;

        // Return the accumulated sums
        (sum_0, sum_1, sum_half)
    }

    pub fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    pub fn compute_state(&mut self) {
        // println!("####### compute state");
        let j_prime: usize;
        let t: usize;
        // println!("num_variables: {:?}", self.num_variables);
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        // println!("l: {:?}", l);
        let j = self.current_round + 1;
        // println!("j: {:?}", j);
        let mut p = 0_usize;
        let s = (j).ilog2() as usize;
        // println!("s: {:?}", s);
        // println!("l - 1: {:?}", l - 1);
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
        // println!("j_prime: {:?}", j_prime);
        // println!("t: {:?}", t);

        if p == 1 {
            self.j_prime_table = vec![
                vec![F::ZERO; Hypercube::stop_value(self.stage_size)];
                Hypercube::stop_value(self.stage_size)
            ];
            let b_num_vars = self.num_variables - j_prime - t;
            for (b_index, _) in Hypercube::new(b_num_vars) {
                // println!("####### X update");

                for (b_prime_index, _) in Hypercube::new(t + 1) {
                    self.x_table[b_prime_index] = F::ZERO;
                    for (x_index, x) in Hypercube::new(j_prime - 1) {
                        let evaluation_point =
                            x_index << (t + 1 + b_num_vars) | b_prime_index << b_num_vars | b_index;
                        // println!("point: {:?}", evaluation_point);
                        let lag_poly = LagrangePolynomial::lag_poly(
                            self.verifier_messages.messages[0..j_prime - 1].to_vec(),
                            self.verifier_messages.message_hats[0..j_prime - 1].to_vec(),
                            x,
                        );
                        // println!(
                        //     "lag_poly: {:?}, eval: {:?}",
                        //     lag_poly,
                        //     self.stream_p.evaluation(evaluation_point)
                        // );
                        self.x_table[b_prime_index] +=
                            lag_poly * self.stream_q.evaluation(evaluation_point);
                        // println!("x_table: {:?}", self.x_table);
                    }
                }

                // println!("####### Y update");

                for (b_prime_prime_index, _) in Hypercube::new(t + 1) {
                    self.y_table[b_prime_prime_index] = F::ZERO;
                    for (x_index, x) in Hypercube::new(j_prime - 1) {
                        let evaluation_point = x_index << (t + 1 + b_num_vars)
                            | b_prime_prime_index << b_num_vars
                            | b_index;
                        // println!("b_prime_prime_index: {:?}", b_prime_prime_index);
                        // println!("point: {:?}", evaluation_point);
                        let lag_poly = LagrangePolynomial::lag_poly(
                            self.verifier_messages.messages[0..j_prime - 1].to_vec(),
                            self.verifier_messages.message_hats[0..j_prime - 1].to_vec(),
                            x,
                        );
                        // println!(
                        //     "lag_poly: {:?}, eval: {:?}",
                        //     lag_poly,
                        //     self.stream_p.evaluation(evaluation_point)
                        // );
                        self.y_table[b_prime_prime_index] +=
                            lag_poly * self.stream_q.evaluation(evaluation_point);
                        // println!("y_table: {:?}", self.y_table);
                    }
                }

                // println!("blendy x_table: {:?}", self.x_table);
                // println!("blendy y_table: {:?}", self.y_table);
                // println!("####### M update");
                for (b_prime_index, _) in Hypercube::new(t + 1) {
                    for (b_prime_prime_index, _) in Hypercube::new(t + 1) {
                        // if (b_prime_index == 0 && b_prime_prime_index == 1) {
                        //     println!("adding to [0][1]: {:?} * {:?}", self.x_table[b_prime_index], self.y_table[b_prime_prime_index]);
                        // }
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
