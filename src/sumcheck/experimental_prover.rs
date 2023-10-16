use ark_ff::Field;
use ark_poly::univariate::SparsePolynomial;
use ark_std::vec::Vec;

use crate::sumcheck::Prover;
use crate::sumcheck::SumcheckMultivariatePolynomial;

// the state of the time prover in the protocol
pub struct ExperimentalProver<F: Field, P: SumcheckMultivariatePolynomial<F>> {
    pub mlp: P, // a polynomial that will be treated as multilinear
    pub mlp_claim: F, // the claimed evaluation of mpl
    pub mlp_evaluated_per_input: Vec<F>,
    pub inter_sums: Vec<F>,
    pub random_challenges: Vec<F>,
    pub current_round: usize,
    pub num_vars: usize,
}

fn bits_to_size<F: Field>(bits: &[F]) -> usize {
    let mut size: usize = 0;
    let mut shift = 0;

    // Iterate through the bits in reverse order (from least significant to most significant)
    for &bit in bits.iter().rev() {
        // If the bit is 1, set the corresponding bit in the size variable
        if bit == F::ONE {
            size |= 1 << shift;
        }
        // Increment the shift value to move to the next bit position
        shift += 1;
    }

    size
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> ExperimentalProver<F, P> {
    // create new time prover state
    pub fn new(mlp: P) -> Self {
        let mlp_claim = mlp.to_evaluations().into_iter().sum();
        let mlp_evaluated_per_input = mlp.to_evaluations();
        let num_vars = mlp.num_vars();
        let mut running_sum = F::ZERO;
        let mut inter_sums = Vec::<F>::with_capacity(num_vars);
        for point_eval in &mlp_evaluated_per_input {
            running_sum += point_eval;
            inter_sums.push(running_sum);
        }
        Self {
            mlp,
            mlp_claim,
            mlp_evaluated_per_input,
            inter_sums,
            random_challenges: Vec::<F>::with_capacity(num_vars),
            current_round: 0,
            num_vars,
        }
    }
}

impl<F: Field, P: SumcheckMultivariatePolynomial<F>> Prover<F> for ExperimentalProver<F, P> {
    // a next-message function using vsbw
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<SparsePolynomial<F>> {
        assert!(self.current_round <= self.total_rounds() - 1, "More rounds than needed."); // self.current_round is zero-indexed
        if self.current_round != 0 {
            self.random_challenges.push(verifier_message.unwrap());
        }

        // TODO: (z-tech) make this better
        let mut free_variables = self.num_vars - self.random_challenges.len();
        if free_variables > 0 {
            free_variables -= 1;
            // println!("free variables: {}", free_variables);
        }

        let mut point_0_range_start = vec![];
        point_0_range_start.extend_from_slice(&self.random_challenges);
        point_0_range_start.extend_from_slice(&vec![F::ZERO]);
        if free_variables > 0 {
            point_0_range_start.extend_from_slice(&vec![F::ZERO; free_variables]);
        }
        let mut point_0_range_end = vec![];
        point_0_range_end.extend_from_slice(&self.random_challenges);
        point_0_range_end.extend_from_slice(&vec![F::ZERO]);
        if free_variables > 0 {
            point_0_range_end.extend_from_slice(&vec![F::ONE; free_variables]);
        }
        let a0 = bits_to_size(&point_0_range_start);
        let b0 = bits_to_size(&point_0_range_end);
        let mut left = F::ZERO;
        if a0 > 0 {
            left = self.inter_sums[a0 - 1];
        }
        let sum_0 = self.inter_sums[b0] - left;
        println!("a0: {}, b0: {}, sum_0: {}", a0, b0, sum_0);

        let mut point_1_range_end = vec![];
        point_1_range_end.extend_from_slice(&self.random_challenges);
        point_1_range_end.extend_from_slice(&vec![F::ONE]);
        if free_variables > 0 {
            point_1_range_end.extend_from_slice(&vec![F::ONE; free_variables]);
        }
        let b1 = bits_to_size(&point_1_range_end);
        let sum_1 = self.inter_sums[b1] - self.inter_sums[b0];
        println!("b1: {}, sums[b1]: {}, sum_1: {}", b1, self.inter_sums[b1], sum_1);

        println!("{}, {}", sum_0, sum_1);
        println!("### ROUND DONE #####");
        // println!("b1: {}, sums[b1]: {}, sum_0: {}, diff: {}", b1, self.inter_sums[b1], sum_0, self.inter_sums[b1] - sum_0);
        // println!("end: {}, start: {}, diff: {}", self.inter_sums[b0], self.inter_sums[a0], self.inter_sums[b0] - self.inter_sums[a0]);
    
        // form a polynomial that s.t. g_round(0) = sum_0, g_round(1) = sum_1
        let g_round: SparsePolynomial<F> = SparsePolynomial::<F>::from_coefficients_vec(vec![(0, sum_0), (1, -sum_0 + sum_1)]);

        // don't forget to increment the round
        self.current_round += 1;
    
        return Some(g_round);
    }
    fn total_rounds(&self) -> usize {
        self.num_vars
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
        DenseMVPolynomial,
        Polynomial,
    };

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;
    type TestPolynomial = multivariate::SparsePolynomial::<TestField, SparseTerm>;

    fn test_polynomial() -> TestPolynomial {
        // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2
        return TestPolynomial::from_coefficients_slice(
            3,
            &[
                (
                    TestField::from(4),
                    multivariate::SparseTerm::new(vec![(0, 1),(1, 1)]),
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
    }

    #[test]
    fn time_prover_init() {
        let prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        assert_eq!(prover.total_rounds(), 3, "should set the number of variables correctly");
    }

    #[test]
    fn time_prover_round_0() {
        // ZEROTH ROUND
        // all variables free
        // 000 = 0
        // 001 = 0
        // 010 = 13
        // 011 = 1
        // sum g0(0) = 14
        // 100 = 2
        // 101 = 2
        // 110 = 0
        // 111 = 7
        // sum g0(1) = 11
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        assert_eq!(g_round_0.evaluate(&TestField::ZERO), TestField::from(14), "g0 should evaluate correctly for input 0");
        assert_eq!(g_round_0.evaluate(&TestField::ONE), TestField::from(11), "g0 should evaluate correctly for input 1");
    }

    #[test]
    fn time_prover_round_1() {
        // FIRST ROUND x0 fixed to 1
        // 101 = 2
        // 100 = 2
        // sum g1(0) = 4
        // 111 = 7
        // 110 = 0
        // sum g1(1) = 7
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        assert_eq!(g_round_0.evaluate(&TestField::ONE), g_round_1.evaluate(&TestField::ZERO) + g_round_1.evaluate(&TestField::ONE));
        assert_eq!(g_round_1.evaluate(&TestField::ZERO), TestField::from(4), "g1 should evaluate correctly for input 0");
        assert_eq!(g_round_1.evaluate(&TestField::ONE), TestField::from(7), "g1 should evaluate correctly for input 1");
    }

    #[test]
    fn time_prover_round_2() {
        // LAST ROUND x1 fixed to 1
        // 110 = 0
        // sum g(0) = 0 
        // 111 = 7
        // sum g(1) = 7
        let mut prover = ExperimentalProver::<TestField, TestPolynomial>::new(test_polynomial());
        let _g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(TestField::ONE)).unwrap(); // x0 fixed to one
        let g_round_2 = prover.next_message(Some(TestField::ONE)).unwrap(); // x1 fixed to one
        assert_eq!(g_round_1.evaluate(&TestField::ONE), g_round_2.evaluate(&TestField::ZERO) + g_round_2.evaluate(&TestField::ONE));
        assert_eq!(g_round_2.evaluate(&TestField::ZERO), TestField::from(0), "g2 should evaluate correctly for input 0");
        assert_eq!(g_round_2.evaluate(&TestField::ONE), TestField::from(7), "g2 should evaluate correctly for input 1");
    }
}