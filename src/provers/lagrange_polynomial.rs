use crate::provers::hypercube::{Hypercube, HypercubeMember};
use ark_ff::{batch_inversion, Field};

pub struct LagrangePolynomial<F: Field> {
    pub messages: Vec<F>,
    pub message_hats: Vec<F>,
    pub message_hat_inverses: Vec<F>,
    pub message_inverses: Vec<F>,
    pub stop_position: usize,
    pub position: usize,
    pub value: F,
}

impl<F: Field> LagrangePolynomial<F> {
    pub fn new(messages: Vec<F>, message_hats: Vec<F>) -> Self {
        // Initialize a stack with capacity for messages/ message_hats and the identity element
        let mut stack: Vec<F> = Vec::with_capacity(messages.len() + 1);
        stack.push(F::ONE);

        // Iterate over the message_hats, update the running product, and push it onto the stack
        let mut running_product: F = F::ONE;
        for message_hat in &message_hats {
            running_product *= message_hat;
            stack.push(running_product);
        }

        // Clone and reverse the messages and message_hats vectors
        let mut messages_clone = messages.clone();
        messages_clone.reverse();
        let mut message_inverses = messages.clone();
        batch_inversion(&mut message_inverses);
        message_inverses.reverse();
        let mut message_hats_clone = message_hats.clone();
        message_hats_clone.reverse();
        let mut message_hat_inverses = message_hats.clone();
        batch_inversion(&mut message_hat_inverses);
        message_hat_inverses.reverse();

        // Return
        Self {
            messages: messages_clone,
            message_hats: message_hats_clone,
            message_hat_inverses,
            message_inverses,
            value: *stack.last().unwrap(),
            stop_position: Hypercube::stop_value(messages.len()),
            position: 0,
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
        // Check if we reached the stop_position
        if self.position >= self.stop_position {
            return None;
        }
        let current_value = self.value;
        let current_position = self.position;
        // Increment
        self.position = Hypercube::next_gray_code(self.position);
        if self.position < self.stop_position {
            let bit_mask = current_position ^ self.position;
            let bit_index = bit_mask.trailing_zeros() as usize;
            let is_mult = current_position & bit_mask == 0;
            self.value = match is_mult {
                true => {
                    self.value * self.message_hat_inverses[bit_index] * self.messages[bit_index]
                }
                false => {
                    self.value * self.message_inverses[bit_index] * self.message_hats[bit_index]
                }
            };
        }
        // Return current value
        Some(current_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{
        hypercube::HypercubeMember, lagrange_polynomial::LagrangePolynomial,
        test_helpers::TestField,
    };

    #[test]
    fn next() {
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut lag_poly: LagrangePolynomial<TestField> =
            LagrangePolynomial::new(messages.clone(), message_hats.clone());
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
