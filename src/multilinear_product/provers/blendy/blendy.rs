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
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        let j = self.current_round + 1;
        let s = (j).ilog2() as usize;
        if j <= l - 1 {
            j_prime = 2_usize.pow(s as u32);
            t = j_prime - 1;
        } else {
            j_prime = l * (j / l);
            t = l - 1;
        }

        // Build this table of lag polys of size j-j_prime
        let r2: Vec<F> = self.verifier_messages.messages[j_prime - 1..].to_vec();
        let mut sequential_lag_poly: LagrangePolynomial<F> =
            LagrangePolynomial::new(VerifierMessages::new(&r2));
        let mut lag_polys: Vec<F> = vec![F::ONE; Hypercube::stop_value(j - j_prime)];
        for (b_prime_prime_index, _) in Hypercube::new(j - j_prime) {
            lag_polys[b_prime_prime_index] = sequential_lag_poly.next().unwrap();
        }

        for (b_prime_index, _) in Hypercube::new(j - j_prime) {
            for (b_prime_prime_index, _) in Hypercube::new(j - j_prime) {
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
                let prod_poly = lag_polys[b_prime_index] * lag_polys[b_prime_prime_index];
                sum_0 = sum_0 * prod_poly;
                sum_1 = sum_1 * prod_poly;
                sum_half = sum_half * prod_poly;
            }
        }
        sum_half = sum_half * self.inverse_four;

        println!("blendy round: {:?}, sum0: {:?}, sum1: {:?}", self.current_round + 1, sum_0, sum_1);

        // Return the accumulated sums
        (sum_0, sum_1, sum_half)
    }

    pub fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    pub fn compute_state(&mut self) {
        let j_prime: usize;
        let t: usize;
        let l = self.num_variables.div_ceil(2 * self.num_stages);
        println!("l: {:?}", l);
        let j = self.current_round + 1;
        println!("j: {:?}", j);
        let mut p = 0_usize;
        let s = ark_std::log2(j) as usize;
        println!("s: {:?}", s);
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
        println!("p: {:?}", p);
        if p == 1 {
            // for convenience while iterating
            let x_num_vars = j_prime - 1;
            let b_prime_num_vars = t + 1;
            println!("num_variables: {:?}, j_prime: {:?}, t: {:?}", self.num_variables, j_prime, t);
            let b_num_vars = ((self.num_variables as i64) - (j_prime as i64) - (t as i64)) as usize;
            let b_and_b_prime_num_of_vars = b_num_vars + b_prime_num_vars;

            // zero out the table
            self.j_prime_table = vec![
                vec![F::ZERO; Hypercube::stop_value(b_prime_num_vars)];
                Hypercube::stop_value(b_prime_num_vars)
            ];

            // let mut sequential_lag_poly: LagrangePolynomial<F> =
            // LagrangePolynomial::new(VerifierMessages::new(&self.verifier_messages.messages[0..x_num_vars].to_vec()));
            // // let mut lag_polys = vec![F::ONE; Hypercube::stop_value(x_num_vars)];
            // for (x_index, _) in Hypercube::new(x_num_vars) {
            //     lag_polys[x_index] = sequential_lag_poly.next().unwrap();
            // }

            for (b_index, _) in Hypercube::new(b_num_vars) {
                // this loop is the same for b_prime and b_prime_prime, so do it in one go
                for (b_prime_index, _) in Hypercube::new(b_prime_num_vars) {
                    // zero out these tables
                    self.x_table[b_prime_index] = F::ZERO;
                    self.y_table[b_prime_index] = F::ZERO;

                    let mut sequential_lag_poly: LagrangePolynomial<F> =
                    LagrangePolynomial::new(VerifierMessages::new(&self.verifier_messages.messages[0..x_num_vars].to_vec()));
                    for (x_index, _) in Hypercube::new(x_num_vars) {
                        let point = x_index << b_and_b_prime_num_of_vars
                            | b_prime_index << b_num_vars
                            | b_index;
                        let lag_poly = sequential_lag_poly.next().unwrap();
                        self.x_table[b_prime_index] +=
                        lag_poly * self.stream_p.get_evaluation(point);
                        self.y_table[b_prime_index] +=
                        lag_poly * self.stream_q.get_evaluation(point);
                    }
                }
                for (b_prime_index, _) in Hypercube::new(b_prime_num_vars) {
                    for (b_prime_prime_index, _) in Hypercube::new(b_prime_num_vars) {
                        self.j_prime_table[b_prime_index][b_prime_prime_index] = self.j_prime_table
                            [b_prime_index][b_prime_prime_index]
                            + (self.x_table[b_prime_index] * self.y_table[b_prime_prime_index]);
                    }
                }
            }

            println!("j_prime 0,0: {:?}", self.j_prime_table[0][0]);
            println!("j_prime 0,1: {:?}", self.j_prime_table[0][1]);
            println!("j_prime 1,0: {:?}", self.j_prime_table[1][0]);
            println!("j_prime 1,1: {:?}", self.j_prime_table[1][1]);
        }
    }
}
