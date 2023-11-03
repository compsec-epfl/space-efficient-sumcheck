use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;
use ark_std::vec::Vec;

use crate::sumcheck::Prover;
use crate::sumcheck::Hypercube;

// the state of the tradeoff prover in the protocol
pub struct TradeoffProver<F: Field> {
    pub claimed_evaluation: F,
    pub current_round: usize,
    pub evaluations: Vec<F>,
    pub num_variables: usize,
    pub verifier_messages: Vec<F>,
}

impl<F: Field> TradeoffProver<F> {
    // class methods
    pub fn lagrange_polynomial(x: &[F], w: &[F]) -> Option<F> {
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
    fn field_elements_to_index(bits: &[F]) -> usize {
        let mut index: usize = 0;

        // Iterate through the bits from most significant to least significant
        for &bit in bits {
            // Shift the index to the left by 1 bit position
            index <<= 1;

            // If the current bit is 1, set the least significant bit of the index to 1
            if bit == F::ONE {
                index |= 1;
            }
        }

        index
    }
    pub fn new(evaluations: Vec<F>, claimed_evaluation: F) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the TradeoffProver instance
        let num_variables: usize = (evaluations.len() as f64).log2() as usize;
        Self {
            claimed_evaluation,
            current_round: 0,
            evaluations,
            num_variables,
            verifier_messages: Vec::<F>::with_capacity(num_variables),
        }
    }
}

impl<F: Field> Prover<F> for TradeoffProver<F> {
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
    }
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, reduce the evaluations table
        if self.current_round != 0 {
            // store the verifier message
            self.verifier_messages.push(verifier_message.unwrap());
        }

        let mut sum_0 = F::ZERO;
        let mut sum_1 = F::ZERO;
        let k = 3;
        let l = self.num_variables / k;
        let s = self.current_round / l;
        let mut precomputed: Vec<F> = vec![F::ZERO; 2_usize.pow(l as u32)];
        for b1 in Hypercube::<F>::new(s*l) {
            let weight: F = TradeoffProver::lagrange_polynomial(
                &b1,
                &self.verifier_messages[0..b1.len()],
            )
            .unwrap();
            for b2 in Hypercube::<F>::new(l) {
                let b2_index = TradeoffProver::field_elements_to_index(&b2);
                for b3 in Hypercube::<F>::new((k-s-1)*l) {
                    let f_index = TradeoffProver::field_elements_to_index(&[b1.clone(), b2.clone(), b3.clone()].concat());
                    precomputed[b2_index] = precomputed[b2_index] + weight * self.evaluations[f_index];
                }
            }
        }

        // compute the range sum lookup over array of b2 values
        let mut partial_sums: Vec<F> = Vec::<F>::with_capacity(precomputed.len());
        let mut running_sum = F::ZERO;
        for eval in &precomputed {
            running_sum += eval;
            partial_sums.push(running_sum);
        }

        // compute the sum
        let j_prime = self.current_round - (s * l);
        for b2_prime in Hypercube::new(j_prime) {
            let weight: F = TradeoffProver::lagrange_polynomial(
                &b2_prime,
                &self.verifier_messages[0..j_prime],
            )
            .unwrap();
            for b2_prime_prime in Hypercube::<F>::new(l - j_prime) {
                let bitmask: usize = 1 << b2_prime_prime.len() - 1;
                let b2_prime_prime_index: usize = TradeoffProver::field_elements_to_index(&[b2_prime.clone(), b2_prime_prime.clone()].concat());
                let is_set: bool = (b2_prime_prime_index & bitmask) != 0;
                println!("prime prime index: {}, bitmask: {}, is_set: {}", b2_prime_prime_index, bitmask, is_set);
                match is_set {
                    false => sum_0 += precomputed[b2_prime_prime_index] * weight,
                    true => sum_1 += precomputed[b2_prime_prime_index] * weight,
                }
            }
        }

        // Form a polynomial s.t. g(0) = sum_0 and g(1) = sum_1
        let g: SparsePolynomial<F> =
            SparsePolynomial::<F>::from_coefficients_vec(vec![(0, sum_0), (1, -sum_0 + sum_1)]);

        // Increment the round counter
        self.current_round += 1;

        // Return the computed polynomial
        return Some(g);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::{
        fields::Fp64,
        fields::{MontBackend, MontConfig},
    };
    use ark_poly::{
        multivariate::{self, SparseTerm, Term},
        DenseMVPolynomial, Polynomial,
    };

    use crate::sumcheck::polynomial::SumcheckMultivariatePolynomial;

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;
    type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

    fn test_polynomial() -> TestPolynomial {
        // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2
        return TestPolynomial::from_coefficients_slice(
            3,
            &[
                (
                    TestField::from(4),
                    multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
                ),
                (
                    TestField::from(7),
                    multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
                ),
                (
                    TestField::from(2),
                    multivariate::SparseTerm::new(vec![(0, 1)]),
                ),
                (
                    TestField::from(13),
                    multivariate::SparseTerm::new(vec![(1, 1)]),
                ),
            ],
        );
    }

    #[test]
    fn init() {
        let test_evaluations = test_polynomial().to_evaluations();
        let prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        assert_eq!(
            prover.total_rounds(),
            3,
            "should set the number of variables correctly"
        );
    }

    #[test]
    fn round_0() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let g_round_0 = prover.next_message(None).unwrap();
        assert_eq!(
            g_round_0.evaluate(&TestField::ZERO),
            TestField::from(14),
            "g0 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_0.evaluate(&TestField::ONE),
            TestField::from(11),
            "g0 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn round_1() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        assert_eq!(
            g_round_0.evaluate(&TestField::ONE),
            g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ZERO),
            TestField::from(4),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            TestField::from(7),
            "g1 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn round_2() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        let g_round_2 = prover.next_message(Some(TestField::ONE)).unwrap(); // x1 fixed to one
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ZERO),
            TestField::from(0),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ONE),
            TestField::from(7),
            "g2 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn outside_hypercube_round_1() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::from(3))).unwrap(); // x0 fixed to 3
        assert_eq!(
            g_round_0.evaluate(&TestField::from(3)),
            g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ZERO),
            TestField::from(12),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&TestField::ONE),
            TestField::from(12),
            "g1 should evaluate correctly for input 1"
        );
    }

    #[test]
    fn outside_hypercube_round_2() {
        let test_evaluations = test_polynomial().to_evaluations();
        let mut prover =
            TradeoffProver::<TestField>::new(test_evaluations.clone(), test_evaluations.iter().sum());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::from(3))).unwrap(); // x0 fixed to 3
        let g_round_2 = prover.next_message(Some(TestField::from(4))).unwrap(); // x1 fixed to 4
        assert_eq!(
            g_round_1.evaluate(&TestField::from(4)),
            g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ZERO),
            TestField::from(11),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&TestField::ONE),
            TestField::from(1),
            "g2 should evaluate correctly for input 1"
        );
    }
}
