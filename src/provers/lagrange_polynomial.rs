use crate::provers::hypercube::{Hypercube, HypercubeMember};
use ark_ff::Field;

use super::verifier_messages::VerifierMessages;

#[derive(Debug)]
pub struct LagrangePolynomial<F: Field> {
    vm: VerifierMessages<F>,
    stop_position: usize,
    last_position: usize,
    position: usize,
    overall_value: F,
    this_iteration_value: F,
    num_skips: usize,
}

impl<F: Field> LagrangePolynomial<F> {
    pub fn new(vm: VerifierMessages<F>) -> Self {
        let mut vm_copy = vm.clone();
        // Iterate over the message_hats, update the running product, and push it onto the stack
        let mut running_product: F = F::ONE;
        for message_hat in &vm.message_hats_partition_1 {
            running_product *= message_hat;
        }

        // these have to be reversed?
        vm_copy.messages.reverse();
        vm_copy.message_hats.reverse();

        vm_copy.messages_partition_1.reverse();
        vm_copy.message_inverses_partition_1.reverse();
        vm_copy.message_hats_partition_1.reverse();
        vm_copy.message_hat_inverses_partition_1.reverse();

        // Return
        Self {
            vm: vm_copy.clone(),
            overall_value: running_product,
            this_iteration_value: running_product,
            stop_position: Hypercube::stop_value(vm.messages.len()),
            position: 0,
            last_position: 0,
            num_skips: 0,
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
    // pub fn _partitioned_lag_poly(x: Vec<F>, x_hat: Vec<F>, b: HypercubeMember) -> F {
    //     let indices_of_zero_or_one: Vec<usize> = x
    //         .iter()
    //         .enumerate()
    //         .filter_map(|(index, &element)| {
    //             if element == F::ZERO || element == F::ONE {
    //                 Some(index)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect();
    //     let mut partition_1: Vec<F> = Vec::with_capacity(x.len() - indices_of_zero_or_one.len());
    //     let mut partition_2: Vec<F> = Vec::with_capacity(indices_of_zero_or_one.len());
    //     let mut partition_1_hat: Vec<F> =
    //         Vec::with_capacity(x.len() - indices_of_zero_or_one.len());
    //     let mut partition_2_hat: Vec<F> = Vec::with_capacity(indices_of_zero_or_one.len());
    //     let mut partitioned = 0;
    //     for (index, element) in x.clone().into_iter().enumerate() {
    //         if partitioned < indices_of_zero_or_one.len()
    //             && index == indices_of_zero_or_one[partitioned]
    //         {
    //             partition_2.push(element);
    //             partition_2_hat.push(*x_hat.get(index).unwrap());
    //             partitioned += 1;
    //         } else {
    //             partition_1.push(element);
    //             partition_1_hat.push(*x_hat.get(index).unwrap());
    //         }
    //     }
    //     let (partition_1_b, partition_2_b) = HypercubeMember::partition(b, indices_of_zero_or_one);
    //     LagrangePolynomial::lag_poly(partition_1, partition_1_hat, partition_1_b)
    //         * LagrangePolynomial::lag_poly(partition_2, partition_2_hat, partition_2_b)
    // }
}

impl<F: Field> Iterator for LagrangePolynomial<F> {
    type Item = F;
    fn next(&mut self) -> Option<Self::Item> {
        println!("### position: {:?}", self.position);
        // Check if we reached the stop_position
        if self.position >= self.stop_position {
            return None;
        }
        if self.position == 0 {
            self.position = Hypercube::next_gray_code(self.position);
            return Some(self.this_iteration_value);
        }

        let last_position = self.last_position;
        let position = self.position;
        // is the contribution of this position zero?
        let s: Vec<bool> = self.vm.messages_partition_2.clone();
        let b: Vec<bool> = HypercubeMember::elements_at_indices(
            HypercubeMember::new(self.vm.messages.len(), self.position).to_vec_bool(),
            self.vm.indices_of_zero_and_ones.clone(),
        );
        if s != b {
            // notice, do not update last position
            self.position = Hypercube::next_gray_code(position);
            return Some(F::ZERO)
        }
        // what changed?
        let bit_diff = position ^ last_position;
        assert!(bit_diff.count_ones() == 1);
        // check if the bit is flipped from false to true, or true to false
        let is_flipped_to_true = position & bit_diff != 0;
        let index_of_flipped_bit = bit_diff.trailing_zeros() as usize;
        // ^ the bit at this index flipped
        let divisor = match is_flipped_to_true {
            // divide out message_hat
            true => self.vm.message_hats[index_of_flipped_bit],
            // divide out by message
            false => self.vm.messages[index_of_flipped_bit],
        };
        let multiplicand = match is_flipped_to_true {
            // multiply by the message
            true => self.vm.messages[index_of_flipped_bit],
            // multiply by the message_hat
            false => self.vm.message_hats[index_of_flipped_bit],
        };
            println!(
                "BEFORE: current: {:?}, overall: {:?}, this: {:?}, divisor: {:?}, multiplicand: {:?}",
                self.this_iteration_value,
                self.overall_value,
                self.this_iteration_value,
                divisor,
                multiplicand,
            );
        self.this_iteration_value = self.this_iteration_value / divisor * multiplicand;
        self.last_position = position;
        self.position = Hypercube::next_gray_code(position);
        Some(self.this_iteration_value)
        
        // let mut current_value = self.this_iteration_value;
        // let current_position = self.position;
        // // Increment
        // self.position = Hypercube::next_gray_code(self.position);
        // let s: Vec<bool> = self.vm.messages_partition_2.clone();
        // let b: Vec<bool> = HypercubeMember::elements_at_indices(
        //     HypercubeMember::new(self.vm.messages.len(), self.position).to_vec_bool(),
        //     self.vm.indices_of_zero_and_ones.clone(),
        // );
        // println!("{:?}, s==b {:?}", self.position, s == b);
        // if self.position < self.stop_position {
        //     // bc of gray code ordering, we expect exactly one bit difference
        //     let bit_diff = current_position ^ self.position;
        //     assert!(bit_diff.count_ones() == 1);
        //     // check if the bit is flipped from false to true, or true to false
        //     let is_flipped_to_true = current_position & bit_diff == 0;
        //     let index_of_flipped_bit = bit_diff.trailing_zeros() as usize;
        //     let divisor = match is_flipped_to_true {
        //         // divide out message_hat
        //         true => {
        //             if self.vm.message_hats[index_of_flipped_bit] == F::ZERO {
        //                 F::ONE
        //             } else {
        //                 self.vm.message_hats[index_of_flipped_bit]
        //             }
        //         }
        //         // divide out by message
        //         false => {
        //             if self.vm.messages[index_of_flipped_bit] == F::ZERO {
        //                 F::ONE
        //             } else {
        //                 self.vm.messages[index_of_flipped_bit]
        //             }
        //         }
        //     };
        //     let multiplicand = match is_flipped_to_true {
        //         // multiply by the message
        //         true => self.vm.messages[index_of_flipped_bit],
        //         // multiply by the message_hat
        //         false => self.vm.message_hats[index_of_flipped_bit],
        //     };
        //     // println!(
        //     //     "self.position: {:?}, index_of_flipped_bit: {:?}",
        //     //     self.position, index_of_flipped_bit
        //     // );
        //     println!(
        //         "BEFORE: current: {:?}, overall: {:?}, this: {:?}, divisor: {:?}, multiplicand: {:?}",
        //         current_value,
        //         self.overall_value,
        //         self.this_iteration_value,
        //         divisor,
        //         multiplicand,
        //     );
        //     if b != s {
        //         self.this_iteration_value = F::ZERO;
        //     } else {
        //         self.overall_value = self.overall_value / divisor * multiplicand;
        //         self.this_iteration_value = self.overall_value;
        //     }
        //     println!(
        //         "AFTER: current: {:?}, overall: {:?}, this: {:?}, divisor: {:?}, multiplicand: {:?}",
        //         current_value,
        //         self.overall_value,
        //         self.this_iteration_value,
        //         divisor,
        //         multiplicand,
        //     );
        // }
        // // Return current value
        // Some(current_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::provers::{
        hypercube::HypercubeMember, lagrange_polynomial::LagrangePolynomial,
        test_helpers::TestField, verifier_messages::VerifierMessages,
    };

    // #[test]
    // fn partitioned_lag_poly() {
    //     let messages: Vec<TestField> =
    //         vec![TestField::from(1), TestField::from(0), TestField::from(7)];
    //     let message_hats: Vec<TestField> = messages
    //         .clone()
    //         .iter()
    //         .map(|message| TestField::from(1) - message)
    //         .collect();
    //     let exp = LagrangePolynomial::lag_poly(
    //         messages.clone(),
    //         message_hats.clone(),
    //         HypercubeMember::new(3, 7),
    //     );
    //     let res = LagrangePolynomial::_partitioned_lag_poly(
    //         messages.clone(),
    //         message_hats.clone(),
    //         HypercubeMember::new(3, 7),
    //     );
    //     assert_eq!(exp, res);
    // }

    #[test]
    fn next() {
        // this is gray code ordering!
        let messages: Vec<TestField> =
            vec![TestField::from(13), TestField::from(0), TestField::from(7)];
        let message_hats: Vec<TestField> = messages
            .clone()
            .iter()
            .map(|message| TestField::from(1) - message)
            .collect();
        let mut vm = VerifierMessages::new();
        vm.receive_message(TestField::from(13));
        vm.receive_message(TestField::from(0));
        vm.receive_message(TestField::from(7));
        let mut lag_poly: LagrangePolynomial<TestField> = LagrangePolynomial::new(vm);
        // let exp_0 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 0),
        // );
        // let actual_0 = lag_poly.next().unwrap();
        // assert_eq!(exp_0, actual_0);

        // let exp_1 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 1),
        // );
        // let actual_1 = lag_poly.next().unwrap();
        // assert_eq!(exp_1, actual_1);

        // let exp_3 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 3),
        // );
        // let actual_3 = lag_poly.next().unwrap();
        // assert_eq!(exp_3, actual_3);

        // let exp_2 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 2),
        // );
        // let actual_2 = lag_poly.next().unwrap();
        // assert_eq!(exp_2, actual_2);

        // let exp_6 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 6),
        // );
        // let actual_6 = lag_poly.next().unwrap();
        // assert_eq!(exp_6, actual_6);

        // let exp_7 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 7),
        // );
        // let actual_7 = lag_poly.next().unwrap();
        // assert_eq!(exp_7, actual_7);

        // let exp_5 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 5),
        // );
        // let actual_5 = lag_poly.next().unwrap();
        // assert_eq!(exp_5, actual_5);

        // let exp_4 = LagrangePolynomial::lag_poly(
        //     messages.clone(),
        //     message_hats.clone(),
        //     HypercubeMember::new(3, 4),
        // );
        // let actual_4 = lag_poly.next().unwrap();
        // assert_eq!(exp_4, actual_4);

        // println!("lag_poly.value: {:?}", lag_poly.value);
        // println!("lag_poly.next(): {:?}", lag_poly.next().unwrap());
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
