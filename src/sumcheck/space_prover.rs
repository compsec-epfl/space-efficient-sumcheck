use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;

use crate::multilinear_extensions::lagrange_polynomial;
use crate::sumcheck::{Bitcube, Hypercube};
use crate::sumcheck::Prover;

// the state of the space prover in the protocol
pub struct SpaceProver<F: Field> {
    pub claimed_evaluation: F, // the claimed evaluation of the multilinear polynomial
    pub evaluations_per_input: Vec<F>, // evaluated values of the multilinear polynomial for each input of the boolean hypercube
    pub random_challenges: Vec<F>,     // random challenges for the protocol
    pub current_round: usize,          // current round of the protocol
    pub num_variables: usize,          // number of variables in the multilinear polynomial
}

impl<F: Field> SpaceProver<F> {
    // create new time prover state
    pub fn new(num_variables: usize, evaluations_per_input: Vec<F>) -> Self {
        // compute the claim
        let claimed_evaluation = evaluations_per_input.iter().sum();
        // return ExperimentalProver instance
        Self {
            claimed_evaluation,
            evaluations_per_input,
            random_challenges: Vec::<F>::with_capacity(num_variables),
            current_round: 0,
            num_variables,
        }
    }
}

fn to_index(point: Vec<bool>) -> usize {
    let mut index: usize = 0;
    for bit in point.iter() {
        index = (index << 1) | if *bit == true { 1 } else { 0 };
    }
    return index;
}

fn to_field_elements<F: Field>(point: Vec<bool>) -> Vec<F> {
    let mut res: Vec<F> = Vec::with_capacity(point.len());
    for bit in point.iter() {
        if *bit == false {
            res.push(F::ZERO);
        } else {
            res.push(F::ONE);
        }
    }
    return res;
}

impl<F: Field> Prover<F> for SpaceProver<F> {
    // a next-message function using cty
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        // Ensure the current round is within bounds
        if self.current_round >= self.total_rounds() {
            return None;
        }

        // If it's not the first round, add the verifier message to random_challenges
        if self.current_round != 0 {
            self.random_challenges.push(verifier_message.unwrap());
        }

        let mut sum_0: F = F::ZERO;
        let mut sum_1: F = F::ZERO;
        for input_start in Bitcube::new(self.current_round) {
            let mask: Vec<F> = to_field_elements(input_start.clone());
            let weight: F = lagrange_polynomial(&mask, &self.random_challenges).unwrap();
            for input_end in Bitcube::new(self.num_variables - input_start.len()) {
                let point_eval = self.evaluations_per_input
                    [to_index([input_start.clone(), input_end.clone()].concat())];
                let update_switch: bool = *input_end.first().unwrap() == false;
                match update_switch {
                    true => sum_0 += point_eval * weight,
                    false => sum_1 += point_eval * weight,
                }
            }
        }

        // form a polynomial that s.t. g_round(0) = sum_0, g_round(1) = sum_1
        let g: SparsePolynomial<F> =
            SparsePolynomial::<F>::from_coefficients_vec(vec![(0, sum_0), (1, -sum_0 + sum_1)]);

        // don't forget to increment the round
        self.current_round += 1;

        return Some(g);
    }
    fn total_rounds(&self) -> usize {
        self.num_variables
    }
    fn num_free_variables(&self) -> usize {
        if self.num_variables == self.random_challenges.len() {
            return 0;
        }
        return self.num_variables - self.random_challenges.len() - 1;
    }
    fn claimed_evaluation(&self) -> F {
        self.claimed_evaluation
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

    use crate::sumcheck::SumcheckMultivariatePolynomial;

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
        let polynomial = test_polynomial();
        let prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
        assert_eq!(
            prover.total_rounds(),
            3,
            "should set the number of variables correctly"
        );
    }

    #[test]
    fn round_0() {
        let polynomial = test_polynomial();
        let mut prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
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
        let polynomial = test_polynomial();
        let mut prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
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
        let polynomial = test_polynomial();
        let mut prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
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
        let polynomial = test_polynomial();
        let mut prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
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
        let polynomial = test_polynomial();
        let mut prover =
            SpaceProver::<TestField>::new(polynomial.num_vars, polynomial.to_evaluations());
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
