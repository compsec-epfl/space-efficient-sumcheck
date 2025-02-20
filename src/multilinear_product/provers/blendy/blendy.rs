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

    pub fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }

    pub fn total_rounds(&self) -> usize {
        self.num_variables
    }

    pub fn compute_round(&self) -> (F, F, F) {
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

        // TODO (z-tech): we can store this between rounds to avoid |r2| mults every round
        let verifier_messages = VerifierMessages::new_from_self(
            &self.verifier_messages,
            j_prime - 1,
            self.verifier_messages.messages.len(),
        );

        // things to help iterating
        let b_prime_num_vars = j - j_prime;
        let v_num_vars: usize = t + j_prime - j - 1;
        let b_prime_index_left_shift = v_num_vars + 1;

        // Outer LagPoly
        let mut sequential_lag_poly_1: LagrangePolynomial<F> =
            LagrangePolynomial::new(verifier_messages.clone());

        // Sums
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let mut sum_half = F::ZERO;
        for (b_prime_index, _) in Hypercube::new(b_prime_num_vars) {
            let lag_poly_1 = sequential_lag_poly_1.next().unwrap();

            // Inner LagPoly
            let mut sequential_lag_poly_2: LagrangePolynomial<F> =
                LagrangePolynomial::new(verifier_messages.clone());
            for (b_prime_prime_index, _) in Hypercube::new(b_prime_num_vars) {
                let lag_poly_2 = sequential_lag_poly_2.next().unwrap();
                for (v_index, _) in Hypercube::new(v_num_vars) {
                    let b_prime_0_v =
                        b_prime_index << b_prime_index_left_shift | 0 << v_num_vars | v_index;
                    let b_prime_prime_0_v =
                        b_prime_prime_index << b_prime_index_left_shift | 0 << v_num_vars | v_index;
                    let b_prime_1_v =
                        b_prime_index << b_prime_index_left_shift | 1 << v_num_vars | v_index;
                    let b_prime_prime_1_v =
                        b_prime_prime_index << b_prime_index_left_shift | 1 << v_num_vars | v_index;
                    sum_0 += lag_poly_1
                        * lag_poly_2
                        * self.j_prime_table[b_prime_0_v][b_prime_prime_0_v];
                    sum_1 += lag_poly_1
                        * lag_poly_2
                        * self.j_prime_table[b_prime_1_v][b_prime_prime_1_v];
                    sum_half += lag_poly_1
                        * lag_poly_2
                        * (self.j_prime_table[b_prime_0_v][b_prime_prime_0_v]
                            + self.j_prime_table[b_prime_0_v][b_prime_prime_1_v]
                            + self.j_prime_table[b_prime_1_v][b_prime_prime_0_v]
                            + self.j_prime_table[b_prime_1_v][b_prime_prime_1_v]);
                }
            }
        }
        sum_half = sum_half * self.inverse_four;
        (sum_0, sum_1, sum_half)
    }

    pub fn compute_state(&mut self) {
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
            let j_prime_table_len = Hypercube::stop_value(t);
            self.j_prime_table = vec![vec![F::ZERO; j_prime_table_len]; j_prime_table_len];

            // some stuff for iterating
            let b_num_vars: usize = n + 1 - j_prime - t;
            let x_num_vars = j_prime - 1;
            let x_index_left_shift = t + b_num_vars;

            for (b_index, _) in Hypercube::new(b_num_vars) {
                for (b_prime_index, _) in Hypercube::new(t) {
                    self.x_table[b_prime_index] = F::ZERO;
                    self.y_table[b_prime_index] = F::ZERO;
                    // LagPoly
                    let mut sequential_lag_poly: LagrangePolynomial<F> =
                        LagrangePolynomial::new(self.verifier_messages.clone());
                    for (x_index, _) in Hypercube::new(x_num_vars) {
                        let evaluation_point =
                            x_index << x_index_left_shift | b_prime_index << b_num_vars | b_index;
                        let lag_poly = sequential_lag_poly.next().unwrap();
                        self.x_table[b_prime_index] +=
                            lag_poly * self.stream_p.evaluation(evaluation_point);
                        self.y_table[b_prime_index] +=
                            lag_poly * self.stream_q.evaluation(evaluation_point);
                    }
                }
                for (b_prime_index, _) in Hypercube::new(t) {
                    for (b_prime_prime_index, _) in Hypercube::new(t) {
                        self.j_prime_table[b_prime_index][b_prime_prime_index] = self.j_prime_table
                            [b_prime_index][b_prime_prime_index]
                            + (self.x_table[b_prime_index] * self.y_table[b_prime_prime_index]);
                    }
                }
            }
        }
    }
}
