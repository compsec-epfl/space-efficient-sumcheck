use ark_ff::Field;

#[derive(Clone, Debug)]
pub struct VerifierMessages<F: Field> {
    pub messages: Vec<F>,
    pub message_hats: Vec<F>, // TODO (z-tech): can probably do this differently in blendy
    pub product_of_message_hats: F,
    pub product_of_message_and_message_hat_inverses: Vec<F>,
    pub product_of_message_hat_and_message_inverses: Vec<F>,
    pub indices_of_zero_and_ones: Vec<usize>,
    pub messages_zeros_and_ones: Vec<bool>,
}

impl<F: Field> VerifierMessages<F> {
    pub fn new(messages: &Vec<F>) -> Self {
        let mut verifier_messages = Self {
            messages: vec![],
            message_hats: vec![],
            product_of_message_hats: F::ONE,
            product_of_message_and_message_hat_inverses: vec![],
            product_of_message_hat_and_message_inverses: vec![],
            indices_of_zero_and_ones: vec![],
            messages_zeros_and_ones: vec![],
        };
        for message in messages {
            verifier_messages.receive_message(*message);
        }
        verifier_messages
    }
    pub fn receive_message(&mut self, message: F) {
        let message_hat = F::ONE - message;
        let message_inverse = if message == F::ZERO {
            F::ONE
        } else {
            message.inverse().unwrap()
        };
        let message_hat_inverse = if message_hat == F::ZERO {
            F::ONE
        } else {
            message_hat.inverse().unwrap()
        };
        self.messages.push(message);
        self.message_hats.push(message_hat);
        self.product_of_message_and_message_hat_inverses
            .push(message * message_hat_inverse);
        self.product_of_message_hat_and_message_inverses
            .push(message_hat * message_inverse);
        if message == F::ZERO || message_hat == F::ZERO {
            self.indices_of_zero_and_ones.push(self.messages.len() - 1);
            self.messages_zeros_and_ones
                .push(if message == F::ONE { true } else { false });
        } else {
            self.product_of_message_hats = self.product_of_message_hats * message_hat;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{test_helpers::TestField, verifier_messages::VerifierMessages};
    use ark_ff::{One, Zero};

    #[test]
    fn receive_message() {
        let mut m0: VerifierMessages<TestField> = VerifierMessages::new(&vec![]);

        // ## receive 13
        m0.receive_message(TestField::from(13));
        assert_eq!(m0.messages, vec![TestField::from(13)]);
        assert_eq!(
            m0.message_hats,
            vec![TestField::one() - TestField::from(13)]
        );
        let empty_indices: Vec<usize> = vec![];
        let empty: Vec<bool> = vec![];
        assert_eq!(m0.indices_of_zero_and_ones, empty_indices);
        assert_eq!(m0.messages_zeros_and_ones, empty);

        // ## receive 0
        m0.receive_message(TestField::zero());
        assert_eq!(m0.messages, vec![TestField::from(13), TestField::zero()]);
        assert_eq!(
            m0.message_hats,
            vec![TestField::one() - TestField::from(13), TestField::one()]
        );
        assert_eq!(m0.indices_of_zero_and_ones, vec![1]);
        assert_eq!(m0.messages_zeros_and_ones, vec![false]);

        // ## receive 7
        m0.receive_message(TestField::from(7));
        assert_eq!(
            m0.messages,
            vec![TestField::from(13), TestField::zero(), TestField::from(7)]
        );
        assert_eq!(
            m0.message_hats,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one(),
                TestField::one() - TestField::from(7)
            ]
        );
        assert_eq!(m0.indices_of_zero_and_ones, vec![1]);
        assert_eq!(m0.messages_zeros_and_ones, vec![false]);

        // ## receive 1
        m0.receive_message(TestField::one());
        assert_eq!(
            m0.messages,
            vec![
                TestField::from(13),
                TestField::zero(),
                TestField::from(7),
                TestField::one()
            ]
        );
        assert_eq!(
            m0.message_hats,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one(),
                TestField::one() - TestField::from(7),
                TestField::zero()
            ]
        );
        assert_eq!(m0.indices_of_zero_and_ones, vec![1, 3]);
        assert_eq!(m0.messages_zeros_and_ones, vec![false, true]);

        let mut m1 = VerifierMessages::new(&vec![]);

        // ## receive zero
        m1.receive_message(TestField::from(0));
        assert_eq!(m1.messages, vec![TestField::from(0)]);
        assert_eq!(m1.message_hats, vec![TestField::one()]);
        assert_eq!(m1.indices_of_zero_and_ones, vec![0]);
        assert_eq!(m1.messages_zeros_and_ones, vec![false]);

        // receive 1
        m1.receive_message(TestField::from(1));
        assert_eq!(m1.messages, vec![TestField::from(0), TestField::one()]);
        assert_eq!(m1.message_hats, vec![TestField::one(), TestField::zero()]);
        assert_eq!(m1.indices_of_zero_and_ones, vec![0, 1]);
        assert_eq!(m1.messages_zeros_and_ones, vec![false, true]);
    }
}
