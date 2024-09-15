use crate::provers::hypercube::{Hypercube, HypercubeMember};
use ark_ff::Field;

use super::verifier_messages::VerifierMessages;

#[derive(Debug)]
pub struct LagrangePolynomial<F: Field> {
    last_position: usize,
    position: usize,
    value: F,
    verifier_messages: VerifierMessages<F>,
    stop_position: usize,
}

impl<F: Field> LagrangePolynomial<F> {
    pub fn lag_poly(x: Vec<F>, x_hat: Vec<F>, b: HypercubeMember) -> F {
        // Iterate over the zipped triple x, x_hat, and boolean hypercube vectors
        x.iter().zip(x_hat.iter()).zip(b).fold(
            // Initial the accumulation to F::ONE
            F::ONE,
            // Closure for the folding operation, taking accumulator and ((x_i, x_hat_i), b_i)
            |acc, ((x_i, x_hat_i), b_i)| {
                // Multiply the accumulator by either x_i or x_hat_i based on the boolean value b_i
                acc * match b_i {
                    true => *x_i,
                    false => *x_hat_i,
                }
            },
        )
    }
    pub fn new(verifier_messages: VerifierMessages<F>) -> Self {
        let num_vars = verifier_messages.messages.len();
        Self {
            last_position: 0,
            position: 0,
            value: verifier_messages.product_of_message_hats,
            verifier_messages,
            stop_position: Hypercube::stop_value(num_vars),
        }
    }
}

impl<F: Field> Iterator for LagrangePolynomial<F> {
    type Item = F;
    fn next(&mut self) -> Option<Self::Item> {
        // Step 1: check if finished iterating
        if self.position >= self.stop_position {
            return None;
        }

        // Step 2: check if this iteration yields zero, in which case we skip processing
        let s: Vec<bool> = self.verifier_messages.messages_zeros_and_ones.clone();
        let b: Vec<bool> = HypercubeMember::elements_at_indices(
            HypercubeMember::new(self.verifier_messages.messages.len(), self.position)
                .to_vec_bool(),
            self.verifier_messages.indices_of_zero_and_ones.clone(),
        );
        if s != b {
            // NOTICE! we do not update last_position in this case
            self.position = Hypercube::next_gray_code(self.position);
            return Some(F::ZERO);
        }

        // Step 3: check if position is 0, which is a special case
        // Notice! step 2 could apply when position == 0
        if self.position == 0 {
            self.position = Hypercube::next_gray_code(self.position);
            return Some(self.value);
        }

        // Step 4: update the value, skip if more than one bit difference
        let bit_diff = self.last_position ^ self.position;
        if bit_diff.count_ones() == 1 {
            let index_of_flipped_bit = bit_diff.trailing_zeros() as usize;
            let is_flipped_to_true = self.position & bit_diff != 0;
            let len = self
                .verifier_messages
                .product_of_message_and_message_hat_inverses
                .len();
            self.value = self.value
                * match is_flipped_to_true {
                    true => {
                        self.verifier_messages
                            .product_of_message_and_message_hat_inverses
                            [len - index_of_flipped_bit - 1]
                    }
                    false => {
                        self.verifier_messages
                            .product_of_message_hat_and_message_inverses
                            [len - index_of_flipped_bit - 1]
                    }
                };
        }

        // Step 5: increment positions
        self.last_position = self.position;
        self.position = Hypercube::next_gray_code(self.position);

        // Step 6: return
        Some(self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::m31::M31;
    use crate::provers::{
        hypercube::HypercubeMember, lagrange_polynomial::LagrangePolynomial,
        test_helpers::TestField, verifier_messages::VerifierMessages,
    };

    #[test]
    fn next() {
        // remember this is gray code ordering!
        let messages: Vec<M31> = vec![M31::from(13_u32), M31::from(0_u32), M31::from(7_u32)];
        let message_hats: Vec<M31> = messages
            .clone()
            .iter()
            .map(|message| M31::from(1_u32) - message)
            .collect();
        let mut lag_poly: LagrangePolynomial<M31> =
            LagrangePolynomial::new(VerifierMessages::new(&vec![
                M31::from(13_u32),
                M31::from(0_u32),
                M31::from(7_u32),
            ]));
        for gray_code_index in [0, 1, 3, 2, 6, 7, 5, 4] {
            let exp = LagrangePolynomial::lag_poly(
                messages.clone(),
                message_hats.clone(),
                HypercubeMember::new(3, gray_code_index),
            );
            assert_eq!(lag_poly.next().unwrap(), exp);
        }
        assert_eq!(lag_poly.next(), None);
    }
    #[test]
    fn boolean_next() {
        // remember this is gray code ordering!
        let messages: Vec<TestField> =
            vec![TestField::from(0), TestField::from(1), TestField::from(1)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut lag_poly: LagrangePolynomial<TestField> =
            LagrangePolynomial::new(VerifierMessages::new(&vec![
                TestField::from(0),
                TestField::from(1),
                TestField::from(1),
            ]));
        for gray_code_index in [0, 1, 3, 2, 6, 7, 5, 4] {
            let exp = LagrangePolynomial::lag_poly(
                messages.clone(),
                message_hats.clone(),
                HypercubeMember::new(3, gray_code_index),
            );
            assert_eq!(lag_poly.next().unwrap(), exp);
        }
        assert_eq!(lag_poly.next(), None);
    }
}
