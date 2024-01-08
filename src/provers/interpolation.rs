use crate::provers::hypercube::Hypercube;
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
        let message_hats: Vec<F> = messages
            .clone()
            .iter()
            .map(|message| F::ONE - message)
            .collect();
        let mut stack: Vec<F> = vec![F::ONE];
        for message_hat in &message_hats {
            stack.push(*stack.last().unwrap() * message_hat);
        }
        Self {
            messages: messages.clone(),
            message_hats,
            stack,
            last_position: None,
        }
    }
}
impl<F: Field> SequentialLagrangePolynomial<F> for BasicSequentialLagrangePolynomial<F> {
    fn next(&mut self) -> F {
        // this is the first call to next() after initialization
        if self.last_position == None {
            self.last_position = Some(0);
            return *self.stack.last().unwrap();
        }
        // check we haven't interated too far
        assert!(self.last_position.unwrap() < Hypercube::<F>::pow2(self.messages.len()) - 1); // e.g. 2 ^ 3 = 8, so 7 is 111
                                                                                              // this is any other next() after initialization
        let last_position = self.last_position.unwrap();
        let next_position = last_position + 1;
        // first, discard what's in the stack up to shared prefix
        let not_shared_bits_in_positions = last_position ^ next_position;
        let index_of_lowest_shared_bit: usize = match not_shared_bits_in_positions.checked_ilog2() {
            Some(index_of_highest_not_shared_bit) => (index_of_highest_not_shared_bit + 1) as usize,
            None => 1, // this can never occur
        };
        for _ in 0..index_of_lowest_shared_bit {
            self.stack.pop();
        }
        // then iterate up to the shared prefix again to compute changes
        for bit_index in (0..index_of_lowest_shared_bit).rev() {
            let next_bit: bool = (next_position >> bit_index) & 1 != 0;
            self.stack.push(match next_bit {
                true => {
                    *self.stack.last().unwrap() * self.messages[self.messages.len() - bit_index - 1]
                }
                false => {
                    *self.stack.last().unwrap()
                        * self.message_hats[self.messages.len() - bit_index - 1]
                }
            })
        }
        self.last_position = Some(next_position);
        return *self.stack.last().unwrap();
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
