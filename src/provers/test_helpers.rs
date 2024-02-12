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

pub fn run_boolean_sumcheck_test<'a, F: Field + std::convert::From<i32>, P: Prover<'a, F>>(
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
    // FIRST ROUND x0 fixed to 0
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

pub fn run_basic_sumcheck_test<'a, F: Field + std::convert::From<i32>, P: Prover<'a, F>>(
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
        Hypercube::new(DenseMVPolynomial::<F>::num_vars(self))
            .map(|point: HypercubeMember| {
                TestHelperPolynomial::<F>::evaluate(
                    self,
                    DenseMVPolynomial::<F>::num_vars(self),
                    point,
                )
                .unwrap()
            })
            .collect()
    }
}

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
