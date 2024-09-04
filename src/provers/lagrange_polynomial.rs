use crate::provers::hypercube::{Hypercube, HypercubeMember};
use ark_ff::Field;

use super::verifier_messages::VerifierMessages;

#[derive(Debug)]
pub struct LagrangePolynomial<F: Field> {
    verifier_messages: VerifierMessages<F>,
    stop_position: usize,
    last_position: usize,
    position: usize,
    value: F,
}

impl<F: Field> LagrangePolynomial<F> {
    pub fn new(verifier_messages_original: VerifierMessages<F>) -> Self {
        let mut verifier_messages = verifier_messages_original.clone();
        // Iterate over the message_hats, update the running product, and push it onto the stack
        let mut value: F = F::ONE;
        for message_hat in &verifier_messages.message_hats_partition_1 {
            value *= message_hat;
        }

        // TODO: forgot why these have to be reversed
        verifier_messages.messages.reverse();
        verifier_messages.message_hats.reverse();

        verifier_messages.messages_partition_1.reverse();
        verifier_messages.message_inverses_partition_1.reverse();
        verifier_messages.message_hats_partition_1.reverse();
        verifier_messages.message_hat_inverses_partition_1.reverse();

        // Return
        Self {
            verifier_messages,
            value,
            stop_position: Hypercube::stop_value(verifier_messages_original.messages.len()),
            position: 0,
            last_position: 0,
        }
    }
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
}

impl<F: Field> Iterator for LagrangePolynomial<F> {
    type Item = F;
    fn next(&mut self) -> Option<Self::Item> {
        println!("### position: {}", self.position);
        // Step 1: check if finished iterating
        if self.position >= self.stop_position {
            return None;
        }

        // Step 2: check if this iteration yields zero, in which case we skip processing
        let s: Vec<bool> = self.verifier_messages.messages_partition_2.clone();
        let b: Vec<bool> = HypercubeMember::elements_at_indices(
            HypercubeMember::new(self.verifier_messages.messages.len(), self.position)
                .to_vec_bool(),
            self.verifier_messages.indices_of_zero_and_ones.clone(),
        );
        if s != b {
            // NOTICE! we do not update last_position in this case
            self.position = Hypercube::next_gray_code(self.position);
            println!("returning zero");
            return Some(F::ZERO);
        }

        // Step 3: check if position is 0, which is a special case
        // Notice! step 2 check could apply to position = 0
        if self.position == 0 {
            self.position = Hypercube::next_gray_code(self.position);
            return Some(self.value);
        }

        // Step 4: compute which bit_index flipped and in which direction, we can skip if more than one bit difference
        let bit_diff = self.last_position ^ self.position;
        if bit_diff.count_ones() == 1 {
            let index_of_flipped_bit = bit_diff.trailing_zeros() as usize;
            let is_flipped_to_true = self.position & bit_diff != 0;
            // Step 5: update the value
            let (divisor, multiplicand) = match is_flipped_to_true {
                true => (
                    if self.verifier_messages.message_hats[index_of_flipped_bit] == F::ZERO {
                        F::ONE
                    } else {
                        self.verifier_messages.message_hats[index_of_flipped_bit]
                    },
                    if self.verifier_messages.messages[index_of_flipped_bit] == F::ZERO {
                        F::ONE
                    } else {
                        self.verifier_messages.messages[index_of_flipped_bit]
                    },
                ),
                false => (
                    self.verifier_messages.messages[index_of_flipped_bit],
                    self.verifier_messages.message_hats[index_of_flipped_bit],
                ),
            };
            self.value = self.value / divisor * multiplicand;
        }

        // Step 6: increment positions
        self.last_position = self.position;
        self.position = Hypercube::next_gray_code(self.position);

        // Step 7: return
        Some(self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{
        hypercube::HypercubeMember, lagrange_polynomial::LagrangePolynomial,
        test_helpers::TestField, verifier_messages::VerifierMessages,
    };

    #[test]
    fn next() {
        // remember this is gray code ordering
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(0), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut lag_poly: LagrangePolynomial<TestField> =
            LagrangePolynomial::new(VerifierMessages::new(&vec![
                TestField::from(13),
                TestField::from(0),
                TestField::from(7),
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
    fn next_boolean() {
        // remember this is gray code ordering
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
