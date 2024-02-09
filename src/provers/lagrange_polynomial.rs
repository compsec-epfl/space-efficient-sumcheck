use crate::provers::hypercube::Hypercube;
use ark_ff::Field;

pub struct LagrangePolynomial<F: Field> {
    pub last_position: Option<usize>,
    pub messages: Vec<F>,
    pub message_hats: Vec<F>,
    pub stack: Vec<F>,
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
        let mut message_hats_clone = message_hats.clone();
        message_hats_clone.reverse();

        // Return
        Self {
            messages: messages_clone,
            message_hats: message_hats_clone,
            stack,
            last_position: None,
        }
    }
    pub fn lag_poly(x: Vec<F>, x_hat: Vec<F>, b: Vec<bool>) -> F {
        // Iterate over the zipped triple x, x_hat, and boolean hypercube vectors
        x.iter().zip(x_hat.iter()).zip(b.iter()).fold(
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
    // Iterator implementation for the struct
    fn next(&mut self) -> Option<Self::Item> {
        // a) Check if this is the first iteration
        if self.last_position == None {
            // Initialize last position
            self.last_position = Some(0);
            // Return the top of the stack
            return Some(*self.stack.last().unwrap());
        }

        // b) Check if in the last iteration we finished iterating
        if self.last_position.unwrap() >= Hypercube::pow2(self.messages.len()) - 1 {
            return None;
        }

        // c) Everything else, first get bit diff
        let last_position = self.last_position.unwrap();
        let next_position = last_position + 1;
        let bit_diff = last_position ^ next_position;

        // Determine the shared prefix of the most significant bits
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;

        // Discard any stack values outside of this prefix
        self.stack.truncate(self.stack.len() - low_index_of_prefix);

        // Iterate up to this prefix computing lag poly correctly
        for bit_index in (0..low_index_of_prefix).rev() {
            let last_element = self.stack.last().unwrap();
            let next_bit: bool = (next_position & (1 << bit_index)) != 0;
            self.stack.push(match next_bit {
                true => *last_element * self.messages[bit_index],
                false => *last_element * self.message_hats[bit_index],
            });
        }

        // Don't forget to update the last position
        self.last_position = Some(next_position);

        // Return the top of the stack
        Some(*self.stack.last().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{lagrange_polynomial::LagrangePolynomial, test_helpers::TestField};

    #[test]
    fn lag_next_test() {
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut bslp: LagrangePolynomial<TestField> =
            LagrangePolynomial::new(messages.clone(), message_hats.clone());
        let st_0: TestField = bslp.next().unwrap();
        let exp_0: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![false, false, false],
        );
        assert_eq!(st_0, exp_0);
        let st_1: TestField = bslp.next().unwrap();
        let exp_1: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![false, false, true],
        );
        assert_eq!(st_1, exp_1);
        let st_2: TestField = bslp.next().unwrap();
        let exp_2: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![false, true, false],
        );
        assert_eq!(st_2, exp_2);
        let st_3: TestField = bslp.next().unwrap();
        let exp_3: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![false, true, true],
        );
        assert_eq!(st_3, exp_3);
        let st_4: TestField = bslp.next().unwrap();
        let exp_4: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![true, false, false],
        );
        assert_eq!(st_4, exp_4);
        let st_5: TestField = bslp.next().unwrap();
        let exp_5: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![true, false, true],
        );
        assert_eq!(st_5, exp_5);
        let st_6: TestField = bslp.next().unwrap();
        let exp_6: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![true, true, false],
        );
        assert_eq!(st_6, exp_6);
        let st_7: TestField = bslp.next().unwrap();
        let exp_7: TestField = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            vec![true, true, true],
        );
        assert_eq!(st_7, exp_7);
        assert_eq!(bslp.next(), None);
    }
}
