use ark_ff::Field;

#[derive(Clone)]
pub struct VerifierMessages<F: Field> {
    // messages and hats
    pub messages: Vec<F>,
    pub message_hats: Vec<F>,
    // inverses
    pub message_inverses: Vec<F>,
    pub message_hat_inverses: Vec<F>,
    // partitions of all (eight additional)
    indices_of_zero_and_ones: Vec<usize>,
    messages_partition_1: Vec<F>,
    messages_partition_2: Vec<F>,
    message_hats_partition_1: Vec<F>,
    message_hats_partition_2: Vec<F>,
    message_inverses_partition_1: Vec<F>,
    message_inverses_partition_2: Vec<F>,
    message_hat_inverses_partition_1: Vec<F>,
    message_hat_inverses_partition_2: Vec<F>,
}

impl<F: Field> VerifierMessages<F> {
    pub fn new() -> Self {
        Self {
            // messages and hats
            messages: vec![],
            message_hats: vec![],
            // inverses
            message_inverses: vec![],
            message_hat_inverses: vec![],
            // partitions of all (eight additional)
            indices_of_zero_and_ones: vec![],
            messages_partition_1: vec![],
            messages_partition_2: vec![],
            message_hats_partition_1: vec![],
            message_hats_partition_2: vec![],
            message_inverses_partition_1: vec![],
            message_inverses_partition_2: vec![],
            message_hat_inverses_partition_1: vec![],
            message_hat_inverses_partition_2: vec![],
        }
    }
    pub fn receive_message(&mut self, message: F) {
        // calculate
        let message_hat = F::ONE - message;
        let message_inverse = F::ONE / message;
        let message_hat_inverse = F::ONE / message_hat;
        // store
        self.messages.push(message);
        self.message_hats.push(message_hat);
        self.message_inverses.push(message_inverse);
        self.message_hat_inverses.push(message_hat_inverse);
        // partition
        if message == F::ZERO || message == F::ONE {
            self.indices_of_zero_and_ones.push(self.messages.len() - 1);
            self.messages_partition_2.push(message);
            self.message_hats_partition_2.push(message_hat);
            self.message_inverses_partition_2.push(message_inverse);
            self.message_hat_inverses_partition_2
                .push(message_hat_inverse);
        } else {
            self.messages_partition_1.push(message);
            self.message_hats_partition_1.push(message_hat);
            self.message_inverses_partition_1.push(message_inverse);
            self.message_hat_inverses_partition_1
                .push(message_hat_inverse);
        }
    }
}
