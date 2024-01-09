use ark_ff::Field;

use crate::provers::hypercube::Hypercube;

pub fn lagrange_polynomial<F: Field>(x: Vec<F>, x_hat: Vec<F>, b: Vec<bool>) -> F {
    x.to_vec()
        .iter()
        .zip(x_hat.iter())
        .zip(b.iter())
        .fold(F::ONE, |acc, ((x_i, x_hat_i), b_i)| {
            acc * match b_i {
                true => x_i,
                false => x_hat_i,
            }
        })
}

pub trait SequentialLagrangePolynomial<F: Field> {
    fn next(&mut self) -> F;
}

pub struct BasicSequentialLagrangePolynomial<F: Field> {
    pub last_position: Option<usize>,
    pub messages: Vec<F>,
    pub message_hats: Vec<F>,
    pub stack: Vec<F>,
}
impl<F: Field> BasicSequentialLagrangePolynomial<F> {
    pub fn new(messages: Vec<F>, message_hats: Vec<F>) -> Self {
        let mut stack: Vec<F> = Vec::with_capacity(messages.len() + 1);
        stack.push(F::ONE);
        for message_hat in message_hats.iter().rev() {
            stack.push(*stack.last().unwrap() * message_hat);
        }
        // return
        Self {
            messages: messages,
            message_hats: message_hats,
            stack,
            last_position: None,
        }
    }
}
impl<F: Field> SequentialLagrangePolynomial<F> for BasicSequentialLagrangePolynomial<F> {
    fn next(&mut self) -> F {
        if self.last_position.is_none() {
            self.last_position = Some(0);
            return *self.stack.last().unwrap();
        }

        // check we haven't interated too far
        assert!(self.last_position.unwrap() < Hypercube::pow2(self.messages.len()) - 1); // e.g. 2 ^ 3 = 8, so 7 is 111

        // this is any other next() after initialization
        let last_position = self.last_position.unwrap();
        let next_position = last_position + 1;
        // first, pop all levels up until shared prefix
        let bit_diff = last_position ^ next_position;
        let low_index_of_prefix = (bit_diff + 1).trailing_zeros() as usize;
        self.stack.truncate(self.stack.len() - low_index_of_prefix);
        // then, iterate up until shared prefix to compute changes
        for bit_index in (0..low_index_of_prefix).rev() {
            let last_element = self.stack.last().unwrap();
            let next_bit: bool = (next_position & (1 << bit_index)) != 0;
            self.stack.push(match next_bit {
                true => *last_element * self.messages[bit_index],
                false => *last_element * self.message_hats[bit_index],
            });
        }
        self.last_position = Some(next_position);
        *self.stack.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{
        interpolation::{
            lagrange_polynomial, BasicSequentialLagrangePolynomial, SequentialLagrangePolynomial,
        },
        test_helpers::TestField,
    };

    #[test]
    fn lag_next_test() {
        let mut messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        messages.reverse();
        let mut message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        message_hats.reverse();
        let mut bslp: BasicSequentialLagrangePolynomial<TestField> =
            BasicSequentialLagrangePolynomial::new(messages.clone(), message_hats.clone());
        let st_0: TestField = bslp.next();
        let exp_0: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![false, false, false],
        );
        assert_eq!(st_0, exp_0);
        let st_1: TestField = bslp.next();
        let exp_1: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![false, false, true],
        );
        assert_eq!(st_1, exp_1);
        let st_2: TestField = bslp.next();
        let exp_2: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![false, true, false],
        );
        assert_eq!(st_2, exp_2);
        let st_3: TestField = bslp.next();
        let exp_3: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![false, true, true],
        );
        assert_eq!(st_3, exp_3);
        let st_4: TestField = bslp.next();
        let exp_4: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![true, false, false],
        );
        assert_eq!(st_4, exp_4);
        let st_5: TestField = bslp.next();
        let exp_5: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![true, false, true],
        );
        assert_eq!(st_5, exp_5);
        let st_6: TestField = bslp.next();
        let exp_6: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![true, true, false],
        );
        assert_eq!(st_6, exp_6);
        let st_7: TestField = bslp.next();
        let exp_7: TestField = lagrange_polynomial(
            messages.clone(),
            message_hats.clone(),
            vec![true, true, true],
        );
        assert_eq!(st_7, exp_7);
    }
}
