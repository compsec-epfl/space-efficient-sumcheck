use crate::provers::hypercube::Hypercube;
use ark_ff::{batch_inversion, Field};
use std::cmp;

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
    pub messages: Vec<F>,
    pub inverse_messages: Vec<F>,
    pub inverse_message_hats: Vec<F>,
    pub last_value: F,
    pub last_position: Option<usize>,
}
impl<F: Field> BasicSequentialLagrangePolynomial<F> {
    pub fn new(messages: Vec<F>) -> Self {
        let last_value: F = messages
            .iter()
            .fold(F::ONE, |acc: F, &x| acc * (F::ONE - x));
        let mut inverse_messages: Vec<F> = messages.clone();
        batch_inversion(&mut inverse_messages);
        let mut inverse_message_hats: Vec<F> = messages
            .clone()
            .iter()
            .map(|message| F::ONE - message)
            .collect();
        batch_inversion(&mut inverse_message_hats);
        Self {
            messages: messages.to_vec(),
            inverse_messages,
            inverse_message_hats,
            last_value,
            last_position: None,
        }
    }
}
impl<F: Field> SequentialLagrangePolynomial<F> for BasicSequentialLagrangePolynomial<F> {
    fn next(&mut self) -> F {
        // this is the first call to next() after initialization
        if self.last_position == None {
            self.last_position = Some(0);
            return self.last_value;
        }

        // check we haven't interated too far
        assert!(self.last_position.unwrap() < Hypercube::pow2(self.messages.len()) - 1); // e.g. 2 ^ 3 = 8, so 7 is 111

        // this is any other next() after initialization
        let last_position = self.last_position.unwrap();
        let next_position = last_position + 1;
        let mut next_value: F = self.last_value;
        // iterate up to the highest order bit to compute changes
        let index_of_highest_set_bit: usize = match last_position == 0 {
            false => cmp::max(last_position.ilog2(), next_position.ilog2()) as usize,
            true => 0, // argument of integer logarithm must be positive
        };
        for bit_index in (0..=index_of_highest_set_bit).rev() {
            let message = self.messages[self.messages.len() - bit_index - 1];
            let message_hat = F::ONE - message;
            let inverse_message = self.inverse_messages[self.messages.len() - bit_index - 1];
            let inverse_message_hat =
                self.inverse_message_hats[self.messages.len() - bit_index - 1];
            let last_bit = (last_position >> bit_index) & 1;
            let next_bit = (next_position >> bit_index) & 1;
            next_value = match (last_bit, next_bit) {
                (0, 1) => next_value * inverse_message_hat * message,
                (1, 0) => next_value * inverse_message * message_hat,
                _ => next_value,
            }
        }
        self.last_value = next_value;
        self.last_position = Some(next_position);
        next_value
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
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        println!("{:?}", message_hats);
        let mut bslp: BasicSequentialLagrangePolynomial<TestField> =
            BasicSequentialLagrangePolynomial::new(messages.clone());
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
