use ark_ff::Field;

#[derive(Clone, Debug)]
pub struct VerifierMessages<F: Field> {
    // messages and hats
    pub messages: Vec<F>,
    pub message_hats: Vec<F>,
    // partitions of all (eight additional)
    pub indices_of_zero_and_ones: Vec<usize>,
    pub messages_partition_2: Vec<bool>,
    // non {0, 1} values
    pub messages_partition_1: Vec<F>,
    pub message_hats_partition_1: Vec<F>,
    pub message_inverses_partition_1: Vec<F>,
    pub message_hat_inverses_partition_1: Vec<F>,
}

impl<F: Field> VerifierMessages<F> {
    pub fn new(messages: &Vec<F>) -> Self {
        let mut verifier_messages = Self {
            // messages and hats
            messages: vec![],
            message_hats: vec![],
            // partitions
            indices_of_zero_and_ones: vec![],
            messages_partition_2: vec![],
            // non {0, 1} values
            messages_partition_1: vec![],
            message_hats_partition_1: vec![],
            message_inverses_partition_1: vec![],
            message_hat_inverses_partition_1: vec![],
        };
        for message in messages {
            verifier_messages.receive_message(*message);
        }
        verifier_messages
    }
    pub fn receive_message(&mut self, message: F) {
        // calculate
        let message_hat = F::ONE - message;
        // store
        self.messages.push(message);
        self.message_hats.push(message_hat);
        // partition
        if message == F::ZERO || message_hat == F::ZERO {
            self.indices_of_zero_and_ones.push(self.messages.len() - 1);
            self.messages_partition_2
                .push(if message == F::ONE { true } else { false });
        } else {
            self.messages_partition_1.push(message);
            self.message_hats_partition_1.push(message_hat);
            self.message_inverses_partition_1.push(F::one() / message);
            self.message_hat_inverses_partition_1
                .push(F::one() / message_hat);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{test_helpers::TestField, verifier_messages::VerifierMessages};
    use ark_ff::{One, Zero};

    #[test]
    fn receive_message_test_1() {
        let mut m = VerifierMessages::new(&vec![]);

        // ## receive 13
        m.receive_message(TestField::from(13));
        // always updated
        assert_eq!(m.messages, vec![TestField::from(13)]);
        assert_eq!(m.message_hats, vec![TestField::one() - TestField::from(13)]);
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![]);
        assert_eq!(m.messages_partition_2, vec![]);
        // otherwise these updated
        assert_eq!(m.messages_partition_1, vec![TestField::from(13)]);
        assert_eq!(
            m.message_hats_partition_1,
            vec![TestField::one() - TestField::from(13)]
        );
        assert_eq!(
            m.message_inverses_partition_1,
            vec![TestField::one() / TestField::from(13)]
        );
        assert_eq!(
            m.message_hat_inverses_partition_1,
            vec![TestField::one() / (TestField::one() - TestField::from(13))]
        );

        // ## receive 0
        m.receive_message(TestField::zero());
        // always updated
        assert_eq!(m.messages, vec![TestField::from(13), TestField::zero()]);
        assert_eq!(
            m.message_hats,
            vec![TestField::one() - TestField::from(13), TestField::one()]
        );
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![1]);
        assert_eq!(m.messages_partition_2, vec![false]);
        // otherwise these updated
        assert_eq!(m.messages_partition_1, vec![TestField::from(13)]);
        assert_eq!(
            m.message_hats_partition_1,
            vec![TestField::one() - TestField::from(13)]
        );
        assert_eq!(
            m.message_inverses_partition_1,
            vec![TestField::one() / TestField::from(13)]
        );
        assert_eq!(
            m.message_hat_inverses_partition_1,
            vec![TestField::one() / (TestField::one() - TestField::from(13))]
        );

        // ## receive 7
        m.receive_message(TestField::from(7));
        // always updated
        assert_eq!(
            m.messages,
            vec![TestField::from(13), TestField::zero(), TestField::from(7)]
        );
        assert_eq!(
            m.message_hats,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one(),
                TestField::one() - TestField::from(7)
            ]
        );
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![1]);
        assert_eq!(m.messages_partition_2, vec![false]);
        // otherwise these updated
        assert_eq!(
            m.messages_partition_1,
            vec![TestField::from(13), TestField::from(7)]
        );
        assert_eq!(
            m.message_hats_partition_1,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one() - TestField::from(7)
            ]
        );
        assert_eq!(
            m.message_inverses_partition_1,
            vec![
                TestField::one() / TestField::from(13),
                TestField::one() / TestField::from(7)
            ]
        );
        assert_eq!(
            m.message_hat_inverses_partition_1,
            vec![
                TestField::one() / (TestField::one() - TestField::from(13)),
                TestField::one() / (TestField::one() - TestField::from(7))
            ]
        );

        // ## receive 1
        m.receive_message(TestField::one());
        // always updated
        assert_eq!(
            m.messages,
            vec![
                TestField::from(13),
                TestField::zero(),
                TestField::from(7),
                TestField::one()
            ]
        );
        assert_eq!(
            m.message_hats,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one(),
                TestField::one() - TestField::from(7),
                TestField::zero()
            ]
        );
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![1, 3]);
        assert_eq!(m.messages_partition_2, vec![false, true]);
        // otherwise these updated
        assert_eq!(
            m.messages_partition_1,
            vec![TestField::from(13), TestField::from(7)]
        );
        assert_eq!(
            m.message_hats_partition_1,
            vec![
                TestField::one() - TestField::from(13),
                TestField::one() - TestField::from(7)
            ]
        );
        assert_eq!(
            m.message_inverses_partition_1,
            vec![
                TestField::one() / TestField::from(13),
                TestField::one() / TestField::from(7)
            ]
        );
        assert_eq!(
            m.message_hat_inverses_partition_1,
            vec![
                TestField::one() / (TestField::one() - TestField::from(13)),
                TestField::one() / (TestField::one() - TestField::from(7))
            ]
        );
    }

    #[test]
    fn receive_message_test_2() {
        let mut m = VerifierMessages::new(&vec![]);

        // ## receive zero
        m.receive_message(TestField::from(0));
        // always updated
        assert_eq!(m.messages, vec![TestField::from(0)]);
        assert_eq!(m.message_hats, vec![TestField::one()]);
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![0]);
        assert_eq!(m.messages_partition_2, vec![false]);
        // otherwise these updated
        assert_eq!(m.messages_partition_1, vec![]);
        assert_eq!(m.message_hats_partition_1, vec![]);
        assert_eq!(m.message_inverses_partition_1, vec![]);
        assert_eq!(m.message_hat_inverses_partition_1, vec![]);

        // receive 1
        m.receive_message(TestField::from(1));
        // always updated
        assert_eq!(m.messages, vec![TestField::from(0), TestField::one()]);
        assert_eq!(m.message_hats, vec![TestField::one(), TestField::zero()]);
        // updated when m in {0,1}
        assert_eq!(m.indices_of_zero_and_ones, vec![0, 1]);
        assert_eq!(m.messages_partition_2, vec![false, true]);
        // otherwise these updated
        assert_eq!(m.messages_partition_1, vec![]);
        assert_eq!(m.message_hats_partition_1, vec![]);
        assert_eq!(m.message_inverses_partition_1, vec![]);
        assert_eq!(m.message_hat_inverses_partition_1, vec![]);
    }
}
