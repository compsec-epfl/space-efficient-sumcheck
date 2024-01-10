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
        let mut stack: Vec<F> = Vec::with_capacity(messages.len() + 1);
        stack.push(F::ONE);
        // didn't notice any perf difference w/ this variable running_product but keeping anyway
        let mut running_product = F::ONE;
        for message_hat in &message_hats {
            running_product *= message_hat;
            stack.push(running_product);
        }
        // confirmed slightly faster to reverse these first rather than index in reverse like v[len - i - 1]
        let mut messages_clone = messages.clone();
        messages_clone.reverse();
        let mut message_hats_clone = message_hats.clone();
        message_hats_clone.reverse();
        // return
        Self {
            messages: messages_clone,
            message_hats: message_hats_clone,
            stack,
            last_position: None,
        }
    }
    pub fn lag_poly(x: Vec<F>, x_hat: Vec<F>, b: Vec<bool>) -> F {
        x.to_vec().iter().zip(x_hat.iter()).zip(b.iter()).fold(
            F::ONE,
            |acc, ((x_i, x_hat_i), b_i)| {
                acc * match b_i {
                    true => x_i,
                    false => x_hat_i,
                }
            },
        )
    }
}
impl<F: Field> Iterator for LagrangePolynomial<F> {
    type Item = F;
    fn next(&mut self) -> Option<Self::Item> {
        // a) check if this is first iteration
        if self.last_position == None {
            // initialize last position
            self.last_position = Some(0);
            // return top of stack
            return Some(*self.stack.last().unwrap());
        }
        // b) check if in last iteration we finished iterating (e.g. 2 ^ 3 = 8, so 7 is 111)
        if self.last_position.unwrap() >= Hypercube::pow2(self.messages.len()) - 1 {
            return None;
        }
        // c) everything else, first get bit diff
        let last_position = self.last_position.unwrap();
        let next_position = last_position + 1;
        let bit_diff = last_position ^ next_position;
        // determine the shared prefix of most significant bits
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;
        // discard any stack values outside of this prefix
        self.stack.truncate(self.stack.len() - low_index_of_prefix);
        // iterate up to this prefix setting computing lag poly correctly
        for bit_index in (0..low_index_of_prefix).rev() {
            let last_element = self.stack.last().unwrap();
            let next_bit: bool = (next_position & (1 << bit_index)) != 0;
            self.stack.push(match next_bit {
                true => *last_element * self.messages[bit_index],
                false => *last_element * self.message_hats[bit_index],
            });
        }
        // don't forget to update last position
        self.last_position = Some(next_position);
        // return top of the stack
        Some(*self.stack.last().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{interpolation::LagrangePolynomial, test_helpers::TestField};

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
