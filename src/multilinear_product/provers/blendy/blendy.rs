use std::time::{Duration, Instant};

use ark_ff::Field;
use ark_std::vec::Vec;

use crate::{
    hypercube::{Hypercube, HypercubeIndices}, interpolation::LagrangePolynomial, messages::VerifierMessages,
    streams::Stream,
};

pub struct BlendyProductProver<F: Field, S: Stream<F>> {
    pub claim: F,
    pub current_round: usize,
    pub streams: Vec<S>,
    pub num_stages: usize,
    pub num_variables: usize,
    pub verifier_messages: VerifierMessages<F>,
    pub verifier_messages_round_comp: VerifierMessages<F>,
    pub x_table: Vec<F>,
    pub y_table: Vec<F>,
    pub j_prime_table: Vec<Vec<F>>,
    pub stage_size: usize,
    pub inverse_four: F,
}

impl<F: Field, S: Stream<F>> BlendyProductProver<F, S> {
    pub fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    pub fn total_rounds(&self) -> usize {
        self.num_variables
    }

    pub fn compute_round(&self) -> (F, F, F) {
        let mut section_1_total = Duration::new(0, 0);
        let mut section_2_total = Duration::new(0, 0);
        let mut section_3_total = Duration::new(0, 0);
        let mut section_4_total = Duration::new(0, 0);
        let section_4 = Instant::now();
        let n = self.num_variables;
        let k = self.num_stages;
        let l = n.div_ceil(2 * k);
        let j = self.current_round + 1;
        let s = j.ilog2();
        let two_pow_s = 1 << s;
        let (j_prime, t) = if j < l {
            let j_prime = two_pow_s;
            let t = std::cmp::min(j_prime, n + 1 - j_prime);
            (j_prime, t)
        } else {
            let j_prime = l * (j / l);
            let t = std::cmp::min(l, n + 1 - j_prime);
            (j_prime, t)
        };

        // things to help iterating
        let b_prime_num_vars = j - j_prime;
        let v_num_vars: usize = t + j_prime - j - 1;
        let b_prime_index_left_shift = v_num_vars + 1;

        // Lag Poly
        let mut sequential_lag_poly: LagrangePolynomial<F> =
            LagrangePolynomial::new(&self.verifier_messages_round_comp);
        let lag_polys_len = Hypercube::stop_value(b_prime_num_vars);
        let mut lag_polys: Vec<F> = vec![F::ONE; lag_polys_len];

        // Sums
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let mut sum_half = F::ZERO;
        let section_3 = Instant::now();
        for b_prime_index in HypercubeIndices::new(b_prime_num_vars) {
            let section_2 = Instant::now();
            for b_prime_prime_index in HypercubeIndices::new(b_prime_num_vars) {
                // doing it like this, for each hypercube member lag_poly is computed exactly once
                if b_prime_index == 0 {
                    lag_polys[b_prime_prime_index] = sequential_lag_poly.next().unwrap();
                }

                let lag_poly_1 = lag_polys[b_prime_index];
                let lag_poly_2 = lag_polys[b_prime_prime_index];
                let lag_poly = lag_poly_1 * lag_poly_2;
                let section_1 = Instant::now();
                for v_index in HypercubeIndices::new(v_num_vars) {
                    let b_prime_0_v =
                        b_prime_index << b_prime_index_left_shift | 0 << v_num_vars | v_index;
                    let b_prime_prime_0_v =
                        b_prime_prime_index << b_prime_index_left_shift | 0 << v_num_vars | v_index;
                    let b_prime_1_v =
                        b_prime_index << b_prime_index_left_shift | 1 << v_num_vars | v_index;
                    let b_prime_prime_1_v =
                        b_prime_prime_index << b_prime_index_left_shift | 1 << v_num_vars | v_index;
                    sum_0 += lag_poly * self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_1 += lag_poly * self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                    sum_half += lag_poly
                        * (self.j_prime_table[b_prime_0_v][b_prime_prime_0_v]
                            + self.j_prime_table[b_prime_0_v][b_prime_prime_1_v]
                            + self.j_prime_table[b_prime_1_v][b_prime_prime_0_v]
                            + self.j_prime_table[b_prime_1_v][b_prime_prime_1_v]);
                }
                section_1_total += section_1.elapsed();
            }
            section_2_total += section_2.elapsed();
        }
        section_3_total += section_3.elapsed();
        sum_half = sum_half * self.inverse_four;
        section_4_total += section_4.elapsed();
        println!("roundcomp_1, {:.4?}", section_1_total.as_millis());
        println!("roundcomp_2, {:.4?}", (section_2_total - section_1_total).as_millis());
        println!("roundcomp_3, {:.4?}", (section_3_total - section_2_total).as_millis());
        println!("roundcomp_4, {:.4?}", (section_4_total - section_3_total).as_millis());
        (sum_0, sum_1, sum_half)
    }

    pub fn compute_state(&mut self) {
        let mut section_1_total = Duration::new(0, 0);
        let mut section_2_total = Duration::new(0, 0);
        let mut section_3_total = Duration::new(0, 0);
        let mut section_4_total = Duration::new(0, 0);
        let section_4 = Instant::now();
        let n = self.num_variables;
        let k = self.num_stages;
        let l = n.div_ceil(2 * k);
        let j = self.current_round + 1;
        let s = j.ilog2();
        let two_pow_s = 1 << s;
        let mut p = false;
        let (j_prime, t) = if j < l {
            if two_pow_s == j {
                p = true;
            }
            let j_prime = two_pow_s;
            let t = std::cmp::min(j_prime, n + 1 - j_prime);
            (j_prime, t)
        } else {
            if j % l == 0 {
                p = true;
            }
            let j_prime = l * (j / l);
            let t = std::cmp::min(l, n + 1 - j_prime);
            (j_prime, t)
        };

        if p {
            // zero out the table
            let table_len = Hypercube::stop_value(t);
            self.j_prime_table = vec![vec![F::ZERO; table_len]; table_len];
            self.x_table = vec![F::ZERO; table_len];
            self.y_table = vec![F::ZERO; table_len];

            // basically, this needs to get "zeroed" out at the beginning of state computation
            self.verifier_messages_round_comp = VerifierMessages::new_from_self(
                &self.verifier_messages,
                j_prime - 1,
                self.verifier_messages.messages.len(),
            );

            // some stuff for iterating
            let b_num_vars: usize = n + 1 - j_prime - t;
            let x_num_vars = j_prime - 1;
            let x_index_left_shift = t + b_num_vars;

            let section_3 = Instant::now();
            for b_index in HypercubeIndices::new(b_num_vars) {
                let section_2 = Instant::now();
                for b_prime_index in HypercubeIndices::new(t) {
                    self.x_table[b_prime_index] = F::ZERO;
                    self.y_table[b_prime_index] = F::ZERO;
                    // LagPoly
                    let mut sequential_lag_poly: LagrangePolynomial<F> =
                        LagrangePolynomial::new(&self.verifier_messages);
                    let section_1 = Instant::now();
                    for x_index in HypercubeIndices::new(x_num_vars) {
                        // I imagine it's this loop taking lots of runtime
                        let evaluation_point =
                            x_index << x_index_left_shift | b_prime_index << b_num_vars | b_index;
                        let lag_poly = sequential_lag_poly.next().unwrap();
                        self.x_table[b_prime_index] +=
                            lag_poly * self.streams[0].evaluation(evaluation_point);
                        self.y_table[b_prime_index] +=
                            lag_poly * self.streams[1].evaluation(evaluation_point);
                    }
                    section_1_total += section_1.elapsed();
                }
                section_2_total += section_2.elapsed();
                for b_prime_index in HypercubeIndices::new(t) {
                    for b_prime_prime_index in HypercubeIndices::new(t) {
                        self.j_prime_table[b_prime_index][b_prime_prime_index] +=
                            self.x_table[b_prime_index] * self.y_table[b_prime_prime_index];
                    }
                }
            }
            section_3_total += section_3.elapsed();
        }
        section_4_total += section_4.elapsed();
        println!("computestate_1, {:.4?}", section_1_total.as_millis());
        println!("computestate_2, {:.4?}", (section_2_total - section_1_total).as_millis());
        println!("computestate_3, {:.4?}", (section_3_total - section_2_total).as_millis());
        println!("computestate_4, {:.4?}", (section_4_total - section_3_total).as_millis());
    }
}
