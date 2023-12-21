use ark_ff::Field;
use ark_std::vec::Vec;
use std::cmp;

use crate::provers::{
    evaluation_stream::EvaluationStream, hypercube::Hypercube, interpolation::lagrange_polynomial,
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
            sums: Vec::<F>::with_capacity(stage_size),
            stage_size,
        }
    }
    fn compute_partial_sums(precomputed: Vec<F>) -> Vec<F> {
        let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(precomputed.len());
        let mut running_sum = F::ZERO;
        for eval in &precomputed {
            running_sum += eval;
            partial_sums.push(running_sum);
        }
        return partial_sums;
    }
    fn lag_init(verifier_messages: &Vec<F>) -> (F, usize) {
        let current_value: F = verifier_messages
            .iter()
            .fold(F::ONE, |acc: F, &x| acc * (F::ONE - x));
        let current_position: usize = 0;
        (current_value, current_position)
    }
    fn lag_next(verifier_messages: &Vec<F>, last_value: F, last_position: usize) -> (F, usize) {
        assert!(last_position < Hypercube::<F>::pow2(verifier_messages.len()) - 1); // e.g. 2 ^ 3 = 8, so 7 is 111
        let next_position: usize = last_position + 1;
        let mut next_value: F = last_value;
        // iterate up to the highest order bit to compute changes
        let index_of_highest_set_bit: usize = match last_position == 0 {
            false => cmp::max(last_position.ilog2(), next_position.ilog2()) as usize,
            true => 0, // argument of integer logarithm must be positive
        };
        for bit_index in (0..=index_of_highest_set_bit).rev() {
            let verifier_message = verifier_messages[verifier_messages.len() - bit_index - 1];
            let verifier_message_hat = F::ONE - verifier_message;
            let last_bit = (last_position >> bit_index) & 1;
            let next_bit = (next_position >> bit_index) & 1;
            next_value = match (last_bit, next_bit) {
                (0, 1) => (next_value / verifier_message_hat) * verifier_message,
                (1, 0) => (next_value / verifier_message) * verifier_message_hat,
                _ => next_value,
            }
        }
        (next_value, next_position)
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
        let mut sum: Vec<F> = vec![F::ZERO; Hypercube::<F>::pow2(b2_num_vars)];
        // 2. Initialize st := LagInit((s - l)l, r)
        let mut lag_poly_st: (F, usize) = Self::lag_init(&self.verifier_messages);
        // 3. For each b1 ∈ {0,1}^(s-1)l
        for b1_index in 0..Hypercube::<F>::pow2(b1_num_vars) {
            // (a) Compute (LagPoly, st) := LagNext(st)
            lag_poly_st = match b1_index == 0 {
                true => lag_poly_st,
                false => Self::lag_next(&self.verifier_messages, lag_poly_st.0, lag_poly_st.1),
            };
            // (b) For each b2 ∈ {0,1}^l, for each b2 ∈ {0,1}^(k-s)l
            for b2_index in 0..Hypercube::<F>::pow2(b2_num_vars) {
                for b3_index in 0..Hypercube::<F>::pow2(b3_num_vars) {
                    let index = b1_index << (b2_num_vars + b3_num_vars)
                        | b2_index << b3_num_vars
                        | b3_index;
                    sum[b2_index] = sum[b2_index]
                        + lag_poly_st.0 * self.evaluation_stream.get_evaluation_from_index(index);
                }
            }
        }
        self.sums = sum;
    }
    fn compute_round(&self) -> (F, F) {
        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let j_prime = self.current_round - (self.current_stage() * self.stage_size); // := j-(s-1)l
        let r_shift = self.current_stage() * self.stage_size;
        for (b2_index, b2) in Hypercube::new(self.stage_size).enumerate() {
            let b2_start: &[F] = &b2[0..(j_prime + 1)];
            let mut r2_start_0: Vec<F> =
                self.verifier_messages[r_shift..(r_shift + j_prime)].to_vec();
            let mut r2_start_1: Vec<F> =
                self.verifier_messages[r_shift..(r_shift + j_prime)].to_vec();
            r2_start_0.push(F::ZERO); // need to add ZERO to end
            r2_start_1.push(F::ONE); // need to add ONE to end
            let lag_poly_0: F = lagrange_polynomial(&b2_start, &r2_start_0).unwrap();
            let lag_poly_1: F = lagrange_polynomial(&b2_start, &r2_start_1).unwrap();
            sum_0 += lag_poly_0 * self.sums[b2_index];
            sum_1 += lag_poly_1 * self.sums[b2_index];
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
        }

        if self.current_round % self.stage_size == 0 {
            self.sum_update();
        }

        // compute the sum
        let sums: (F, F) = self.compute_round();

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
        interpolation::lagrange_polynomial,
        test_helpers::{
            run_basic_sumcheck_test, test_polynomial, BasicEvaluationStream, TestField,
        },
        TradeoffProver,
    };
    use ark_ff::Field;

    #[test]
    fn sumcheck() {
        let evaluation_stream: BasicEvaluationStream<TestField> =
            BasicEvaluationStream::new(test_polynomial());
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 1));
        run_basic_sumcheck_test(TradeoffProver::new(Box::new(&evaluation_stream), 3));
    }

    #[test]
    fn lag_init_test() {
        let verifier_messages: Vec<TestField> = vec![
            TestField::from(13),
            TestField::from(11),
            TestField::from(7),
            TestField::from(2),
        ];
        let lag_poly_st: (TestField, usize) = TradeoffProver::lag_init(&verifier_messages);
        assert_eq!(
            lag_poly_st.0,
            lagrange_polynomial(
                &vec![
                    TestField::ZERO,
                    TestField::ZERO,
                    TestField::ZERO,
                    TestField::ZERO
                ],
                &verifier_messages
            )
            .unwrap()
        );
        assert_eq!(lag_poly_st.1, 0);
    }

    #[test]
    fn lag_next_test() {
        let verifier_messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let st_0: (TestField, usize) = TradeoffProver::lag_init(&verifier_messages);
        let st_1: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_0.0, st_0.1);
        assert_eq!(
            st_1.0,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ZERO, TestField::ONE],
                &verifier_messages
            )
            .unwrap()
        );
        let st_2: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_1.0, st_1.1);
        assert_eq!(
            st_2.0,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ONE, TestField::ZERO],
                &verifier_messages
            )
            .unwrap()
        );
        let st_3: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_2.0, st_2.1);
        assert_eq!(
            st_3.0,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ONE, TestField::ONE],
                &verifier_messages
            )
            .unwrap()
        );
        let st_4: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_3.0, st_3.1);
        assert_eq!(
            st_4.0,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ZERO, TestField::ZERO],
                &verifier_messages
            )
            .unwrap()
        );
        let st_5: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_4.0, st_4.1);
        assert_eq!(
            st_5.0,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ZERO, TestField::ONE],
                &verifier_messages
            )
            .unwrap()
        );
        let st_6: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_5.0, st_5.1);
        assert_eq!(
            st_6.0,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ONE, TestField::ZERO],
                &verifier_messages
            )
            .unwrap()
        );
        let st_7: (TestField, usize) = TradeoffProver::lag_next(&verifier_messages, st_1.0, st_1.1);
        assert_eq!(
            st_7.0,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ONE, TestField::ONE],
                &verifier_messages
            )
            .unwrap()
        );
    }
}
