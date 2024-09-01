use crate::provers::hypercube::{Hypercube, HypercubeMember};
use ark_ff::{batch_inversion, Field};

use super::verifier_messages::VerifierMessages;

pub struct LagrangePolynomial<F: Field> {
    vm: VerifierMessages<F>,
    stop_position: usize,
    position: usize,
    value: F,
}

impl<F: Field> LagrangePolynomial<F> {
    pub fn new(mut vm: VerifierMessages<F>, _messages: Vec<F>, _message_hats: Vec<F>) -> Self {
        // Initialize a stack with capacity for messages/ message_hats and the identity element
        let mut stack: Vec<F> = Vec::with_capacity(vm.messages.len() + 1);
        stack.push(F::ONE);

        // Iterate over the message_hats, update the running product, and push it onto the stack
        let mut running_product: F = F::ONE;
        for message_hat in &vm.message_hats {
            running_product *= message_hat;
            stack.push(running_product);
        }

        // Clone and reverse the messages and message_hats vectors
        let mut messages_clone = vm.messages.clone();
        messages_clone.reverse();
        let mut message_inverses = vm.messages.clone();
        batch_inversion(&mut message_inverses);
        message_inverses.reverse();
        let mut message_hats_clone = vm.message_hats.clone();
        message_hats_clone.reverse();
        let mut message_hat_inverses = vm.message_hats.clone();
        batch_inversion(&mut message_hat_inverses);
        message_hat_inverses.reverse();

        vm.messages.reverse();
        vm.message_inverses.reverse();
        vm.message_hats.reverse();
        vm.message_hat_inverses.reverse();

        assert_eq!(messages_clone, vm.messages);
        assert_eq!(message_inverses, vm.message_inverses);
        assert_eq!(message_hats_clone, vm.message_hats);
        assert_eq!(message_hat_inverses, vm.message_hat_inverses);

        // Return
        Self {
            vm: vm.clone(),
            value: *stack.last().unwrap(),
            stop_position: Hypercube::stop_value(vm.messages.len()),
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
    pub fn partitioned_lag_poly(x: Vec<F>, x_hat: Vec<F>, b: HypercubeMember) -> F {
        let indices_of_zero_or_one: Vec<usize> = x
            .iter()
            .enumerate()
            .filter_map(|(index, &element)| {
                if element == F::ZERO || element == F::ONE {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        let mut partition_1: Vec<F> = Vec::with_capacity(x.len() - indices_of_zero_or_one.len());
        let mut partition_2: Vec<F> = Vec::with_capacity(indices_of_zero_or_one.len());
        let mut partition_1_hat: Vec<F> =
            Vec::with_capacity(x.len() - indices_of_zero_or_one.len());
        let mut partition_2_hat: Vec<F> = Vec::with_capacity(indices_of_zero_or_one.len());
        let mut partitioned = 0;
        for (index, element) in x.clone().into_iter().enumerate() {
            if partitioned < indices_of_zero_or_one.len()
                && index == indices_of_zero_or_one[partitioned]
            {
                partition_2.push(element);
                partition_2_hat.push(*x_hat.get(index).unwrap());
                partitioned += 1;
            } else {
                partition_1.push(element);
                partition_1_hat.push(*x_hat.get(index).unwrap());
            }
        }
        let (partition_1_b, partition_2_b) = HypercubeMember::partition(b, indices_of_zero_or_one);
        LagrangePolynomial::lag_poly(partition_1, partition_1_hat, partition_1_b)
            * LagrangePolynomial::lag_poly(partition_2, partition_2_hat, partition_2_b)
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
                    self.value * self.vm.message_hat_inverses[bit_index] * self.vm.messages[bit_index]
                }
                false => {
                    self.value * self.vm.message_inverses[bit_index] * self.vm.message_hats[bit_index]
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
        test_helpers::TestField, verifier_messages::VerifierMessages,
    };

    #[test]
    fn partitioned_lag_poly() {
        let messages: Vec<TestField> =
            vec![TestField::from(1), TestField::from(0), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let exp = LagrangePolynomial::lag_poly(
            messages.clone(),
            message_hats.clone(),
            HypercubeMember::new(3, 7),
        );
        let res = LagrangePolynomial::partitioned_lag_poly(
            messages.clone(),
            message_hats.clone(),
            HypercubeMember::new(3, 7),
        );
        assert_eq!(exp, res);
    }

    #[test]
    fn next() {
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(11), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut vm = VerifierMessages::new();
        vm.receive_message(TestField::from(13));
        vm.receive_message(TestField::from(11));
        vm.receive_message(TestField::from(7));
        let mut lag_poly: LagrangePolynomial<TestField> =
            LagrangePolynomial::new(vm, messages.clone(), message_hats.clone());
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
