use ark_ff::Field;

pub fn lagrange_polynomial<F: Field>(x: &[F], w: &[F]) -> Option<F> {
    if x.len() != w.len() {
        None
    } else {
        Some(
            x.to_vec()
                .iter()
                .zip(w.iter())
                .fold(F::ONE, |acc, (&x_i, &w_i)| {
                    acc * (x_i * w_i + (F::ONE - x_i) * (F::ONE - w_i))
                }),
        )
    }
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
    pub fn new(messages: Vec<F>) -> Self {
        let mut stack: Vec<F> = Vec::with_capacity(messages.len() + 1);
        stack.push(F::ONE);
        for message in &messages {
            stack.push(*stack.last().unwrap() * (F::ONE - message));
        }
        let mut messages_clone = messages.clone();
        messages_clone.reverse();
        let message_hats: Vec<F> = messages_clone
            .clone()
            .iter()
            .map(|message| F::ONE - message)
            .collect();
        Self {
            messages: messages_clone,
            message_hats,
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
    use ark_ff::Field;

    #[test]
    fn lag_next_test() {
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let mut bslp: BasicSequentialLagrangePolynomial<TestField> =
            BasicSequentialLagrangePolynomial::new(messages.clone());
        let st_0: TestField = bslp.next();
        assert_eq!(
            st_0,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ZERO, TestField::ZERO],
                &messages
            )
            .unwrap()
        );
        let st_1: TestField = bslp.next();
        assert_eq!(
            st_1,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ZERO, TestField::ONE],
                &messages
            )
            .unwrap()
        );
        let st_2: TestField = bslp.next();
        assert_eq!(
            st_2,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ONE, TestField::ZERO],
                &messages
            )
            .unwrap()
        );
        let st_3: TestField = bslp.next();
        assert_eq!(
            st_3,
            lagrange_polynomial(
                &vec![TestField::ZERO, TestField::ONE, TestField::ONE],
                &messages
            )
            .unwrap()
        );
        let st_4: TestField = bslp.next();
        assert_eq!(
            st_4,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ZERO, TestField::ZERO],
                &messages
            )
            .unwrap()
        );
        let st_5: TestField = bslp.next();
        assert_eq!(
            st_5,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ZERO, TestField::ONE],
                &messages
            )
            .unwrap()
        );
        let st_6: TestField = bslp.next();
        assert_eq!(
            st_6,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ONE, TestField::ZERO],
                &messages
            )
            .unwrap()
        );
        let st_7: TestField = bslp.next();
        assert_eq!(
            st_7,
            lagrange_polynomial(
                &vec![TestField::ONE, TestField::ONE, TestField::ONE],
                &messages
            )
            .unwrap()
        );
    }
}
