use crate::provers::{
    evaluation_stream::EvaluationStream,
    hypercube::{Hypercube, HypercubeMember},
    prover::Prover,
};
use ark_ff::{
    fields::{Fp64, MontBackend, MontConfig},
    Field,
};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    DenseMVPolynomial,
};

#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]

pub struct TestFieldConfig;
pub type TestField = Fp64<MontBackend<TestFieldConfig, 1>>;
pub type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

pub fn run_boolean_sumcheck_test<
    'a,
    F: Field + std::convert::From<i32>,
    S: EvaluationStream<F>,
    P: Prover<'a, F, S>,
>(
    mut prover: P,
) {
    // ZEROTH ROUND
    // all variables free
    // 000 = 0
    // 001 = 0
    // 010 = 13
    // 011 = 1
    // sum g0(0) = 14
    // 100 = 2
    // 110 = 0
    // 101 = 2
    // 111 = 7
    // sum g0(1) = 11
    let round_0 = prover.next_message(None).unwrap();
    assert_eq!(
        round_0.0,
        F::from(14),
        "g0 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_0.1,
        F::from(11),
        "g0 should evaluate correctly for input 1"
    );
    // FIRST ROUND x0 fixed to 1
    // 101 = 2
    // 100 = 2
    // sum g1(0) = 4
    // 111 = 7
    // 110 = 0
    // sum g1(1) = 7
    let round_1 = prover.next_message(Some(F::ONE)).unwrap(); // x0 fixed to one
    assert_eq!(round_0.1, round_1.0 + round_1.1);
    assert_eq!(
        round_1.0,
        F::from(4),
        "g1 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_1.1,
        F::from(7),
        "g1 should evaluate correctly for input 1"
    );
    // LAST ROUND x1 fixed to 1
    // 110 = 0
    // sum g(0) = 0
    // 111 = 7
    // sum g(1) = 7
    let round_2 = prover.next_message(Some(F::ONE)).unwrap(); // x1 fixed to one
    assert_eq!(round_1.1, round_2.0 + round_2.1);
    assert_eq!(
        round_2.0,
        F::from(0),
        "g2 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(7),
        "g2 should evaluate correctly for input 1"
    );
}

pub fn run_basic_sumcheck_test<
    'a,
    F: Field + std::convert::From<i32>,
    S: EvaluationStream<F>,
    P: Prover<'a, F, S>,
>(
    mut prover: P,
) {
    // FIRST ROUND x0 fixed to 3
    // 3,0,1 = 6
    // 3,0,0 = 6
    // sum g1(0) = 12
    // 3,1,1 = 38 = 0 mod 19
    // 3,1,0 = 31 = 12 mod 19
    // sum g1(1) = 12
    let round_0 = prover.next_message(None).unwrap();
    let round_1 = prover.next_message(Some(F::from(3))).unwrap(); // x0 fixed to 3
    assert_eq!(
        round_0.0 - (round_0.0 - round_0.1) * F::from(3),
        round_1.0 + round_1.1
    );
    assert_eq!(
        round_1.0,
        F::from(12),
        "g1 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_1.1,
        F::from(12),
        "g1 should evaluate correctly for input 1"
    );
    // LAST ROUND x1 fixed to 4
    // 3,4,0 = 108 = 11 mod 19
    // sum g(0) = 11
    // 3,4,1 = 134 = 1 mod 19
    // sum g(1) = 1
    let round_2 = prover.next_message(Some(F::from(4))).unwrap(); // x1 fixed to 4
    assert_eq!(
        round_1.0 - (round_1.0 - round_1.1) * F::from(4),
        round_2.0 + round_2.1
    );
    assert_eq!(
        round_2.0,
        F::from(11),
        "g2 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(1),
        "g2 should evaluate correctly for input 1"
    );
}

pub fn run_product_sumcheck_test<
    'a,
    F: Field + std::convert::From<i32>,
    S: EvaluationStream<F>,
    P: Prover<'a, F, S>,
>(
    mut prover: P,
) {
    // ZEROTH ROUND
    // 0000 = 0 * 0 = 0
    // 0001 = 1 * 1 = 1
    // 0010 = 0 * 0 = 0
    // 0011 = 1 * 1 = 1
    // 0100 = 13 * 13 = 17
    // 0101 = 14 * 14 = 6
    // 0110 = 1 * 1 = 1
    // 0111 = 2 * 2 = 4
    // sum g0(0) = 11
    // 1000 = 2 * 2 = 4
    // 1001 = 3 * 3 = 9
    // 1010 = 2 * 2 = 4
    // 1011 = 3 * 3 = 9
    // 1100 = 0 * 0 = 0
    // 1101 = 1 * 1 = 1
    // 1110 = 7 * 7 = 11
    // 1111 = 8 * 8 = 7
    // sum g0(1) = 7
    let round_0 = prover.next_message(None).unwrap();
    assert_eq!(
        round_0.0,
        F::from(11),
        "g0 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_0.1,
        F::from(7),
        "g0 should evaluate correctly for input 1"
    );
    // FIRST ROUND x0 fixed to 3
    // 3000 = (0 * 17) + (2 * 3) = 6 * 6 = 17
    // 3001 = (1 * 17) + (3 * 3) = 7 * 7 = 11
    // 3010 = (0 * 17) + (2 * 3) = 6 * 6 = 17
    // 3011 = (1 * 17) + (3 * 3) = 7 * 7 = 11
    // sum g1(0) = 18
    // 3100 = (13 * 17) + (0 * 3) = 12 * 12 = 11
    // 3101 = (14 * 17) + (1 * 3) = 13 * 13 = 17
    // 3110 = (1 * 17) + (7 * 3) = 0 * 0 = 0
    // 3111 = (2 * 17) + (8 * 3) = 1 * 1 = 1
    // sum g1(1) = 10
    let round_1 = prover.next_message(Some(F::from(3))).unwrap(); // x0 fixed to 3
    // assert_eq!(
    //     round_0.0 - (round_0.0 - round_0.1) * F::from(3) * F::from(3),
    //     round_1.0 + round_1.1
    // );
    assert_eq!(
        round_1.0,
        F::from(18),
        "g1 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_1.1,
        F::from(10),
        "g1 should evaluate correctly for input 1"
    );
    // SECOND ROUND x1 fixed to 4
    // 3400 = (6 * 16) + (12 * 4) = 11 * 11 = 7
    // 3401 = (7 * 16) + (13 * 4) = 12 * 12 = 11
    // sum g2(0) = 18
    // 3410 = (6 * 16) + (0 * 4) = 1 * 1 = 1
    // 3411 = (7 * 16) + (1 * 4) = 2 * 2 = 4
    // sum g2(1) = 5
    let round_2 = prover.next_message(Some(F::from(4))).unwrap(); // x1 fixed to 4
    // assert_eq!(
    //     round_1.0 - (round_1.0 - round_1.1) * F::from(4),
    //     round_2.0 + round_2.1
    // );
    assert_eq!(
        round_2.0,
        F::from(18),
        "g2 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(5),
        "g2 should evaluate correctly for input 1"
    );
    // LAST ROUND x2 fixed to 7
    // 3470 = (11 * 13) + (1 * 7) = 17 * 17 = 4
    // sum g3(0) = 4
    // 3471 = (12 * 13) + (2 * 7) = 18 * 18 = 1
    // sum g3(1) =
    let round_2 = prover.next_message(Some(F::from(7))).unwrap(); // x2 fixed to 7
    // assert_eq!(
    //     round_1.0 - (round_1.0 - round_1.1) * F::from(4),
    //     round_2.0 + round_2.1
    // );
    assert_eq!(
        round_2.0,
        F::from(4),
        "g3 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(1),
        "g3 should evaluate correctly for input 1"
    );
}

pub fn test_polynomial() -> Vec<TestField> {
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
    )
    .to_evaluations();
}

pub fn test_polynomial_2() -> Vec<TestField> {
    // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2 + 1x_4
    return TestPolynomial::from_coefficients_slice(
        4,
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
            (
                TestField::from(1),
                multivariate::SparseTerm::new(vec![(3, 1)]),
            ),
        ],
    )
    .to_evaluations();
}

// https://github.com/montekki/thaler-study/blob/master/sum-check-protocol/src/lib.rs
pub trait TestHelperPolynomial<F: Field> {
    fn evaluate(&self, num_variables: usize, point: HypercubeMember) -> Option<F>;
    fn num_vars(&self) -> usize;
    fn to_evaluations(&self) -> Vec<F>;
}
impl<F: Field> TestHelperPolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, num_variables: usize, point: HypercubeMember) -> Option<F> {
        let mut point_field_element: Vec<F> = Vec::with_capacity(num_variables);
        for bit in point {
            point_field_element.push(match bit {
                true => F::ONE,
                false => F::ZERO,
            })
        }
        let mut eval = F::ZERO;
        for (coeff, term) in self.terms().iter() {
            eval += term.evaluate(&point_field_element) * coeff;
        }
        Some(eval)
    }
    fn num_vars(&self) -> usize {
        DenseMVPolynomial::num_vars(self)
    }
    fn to_evaluations(&self) -> Vec<F> {
        let num_vars = DenseMVPolynomial::<F>::num_vars(self);
        let mut evaluations = vec![];
        for index in 0..Hypercube::stop_value(num_vars) {
            evaluations.push(
                TestHelperPolynomial::<F>::evaluate(
                    self,
                    num_vars,
                    HypercubeMember::new(num_vars, index),
                )
                .unwrap(),
            );
        }
        evaluations
    }
}

#[derive(Debug)]
pub struct BasicEvaluationStream<F: Field> {
    pub evaluations: Vec<F>,
    pub num_variables: usize,
}

impl<F: Field> BasicEvaluationStream<F> {
    pub fn new(evaluations: Vec<F>) -> Self {
        // abort if length not a power of two
        assert_eq!(
            evaluations.len() != 0 && evaluations.len().count_ones() == 1,
            true
        );
        // return the BasicEvaluationStream instance
        let num_variables: usize = evaluations.len().ilog2() as usize;
        Self {
            evaluations,
            num_variables,
        }
    }
    pub fn vec_of_field_to_usize(vec: Vec<F>) -> usize {
        // Reverse the vector to start from the least significant bit
        let reversed_vec: Vec<F> = vec.into_iter().rev().collect();

        // Calculate the decimal value
        let decimal_value: usize = reversed_vec
            .iter()
            .enumerate()
            .filter(|(_, &bit)| bit == F::ONE)
            .map(|(i, _)| 2usize.pow(i as u32))
            .sum();

        decimal_value
    }
}

impl<F: Field> EvaluationStream<F> for BasicEvaluationStream<F> {
    fn get_claimed_sum(&self) -> F {
        self.evaluations.iter().sum()
    }
    fn get_evaluation(&self, point: usize) -> F {
        self.evaluations[point]
    }
    fn get_num_variables(&self) -> usize {
        self.evaluations.len().ilog2() as usize
    }
}

// BenchEvaluationStream just returns the field value of the index and uses constant memory
#[derive(Debug)]
pub struct BenchEvaluationStream<F: Field> {
    pub num_variables: usize,
    pub claimed_sum: F,
}
impl<F: Field> BenchEvaluationStream<F> {
    pub fn new(num_variables: usize) -> Self {
        let hypercube_len = 2usize.pow(num_variables.try_into().unwrap());
        let mut claimed_sum: F = F::ZERO;
        for i in 0..hypercube_len {
            claimed_sum += F::from(i as u64);
        }
        Self {
            num_variables,
            claimed_sum,
        }
    }
    pub fn vec_of_field_to_usize(vec: Vec<F>) -> usize {
        // Reverse the vector to start from the least significant bit
        let reversed_vec: Vec<F> = vec.into_iter().rev().collect();

        // Calculate the decimal value
        let decimal_value: usize = reversed_vec
            .iter()
            .enumerate()
            .filter(|(_, &bit)| bit == F::ONE)
            .map(|(i, _)| 2usize.pow(i as u32))
            .sum();

        decimal_value
    }
}
impl<F: Field> EvaluationStream<F> for BenchEvaluationStream<F> {
    fn get_claimed_sum(&self) -> F {
        self.claimed_sum
    }
    fn get_evaluation(&self, point: usize) -> F {
        F::from(point as u64)
    }
    fn get_num_variables(&self) -> usize {
        self.num_variables
    }
}
