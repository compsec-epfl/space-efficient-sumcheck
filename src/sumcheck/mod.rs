pub mod hypercube;
pub mod proof;
pub mod prover;

pub mod experimental_prover;
pub mod space_prover;
pub mod time_prover;
pub mod tradeoff_prover;

pub use hypercube::{Bitcube, Hypercube};
pub use proof::Sumcheck;
pub use prover::Prover;

pub use experimental_prover::ExperimentalProver;
pub use space_prover::SpaceProver;
pub use time_prover::TimeProver;
pub use tradeoff_prover::TradeoffProver;

#[cfg(test)]
pub mod unit_test_helpers {
    use ark_ff::{
        fields::Fp64,
        fields::{MontBackend, MontConfig},
        Field, Zero,
    };
    use ark_poly::{
        multivariate::{self, SparseTerm, Term},
        univariate, DenseMVPolynomial, Polynomial,
    };

    use super::{Hypercube, Prover};

    // https://github.com/montekki/thaler-study/blob/master/sum-check-protocol/src/lib.rs
    pub trait SumcheckMultivariatePolynomial<F: Field> {
        fn evaluate(&self, point: &[F]) -> Option<F>;
        fn fix_variables(&self, partial_point: &[F]) -> Self;
        fn to_univariate(&self) -> univariate::SparsePolynomial<F>;
        fn num_vars(&self) -> usize;
        fn to_evaluations(&self) -> Vec<F>;
    }

    impl<F: Field> SumcheckMultivariatePolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
        fn evaluate(&self, point: &[F]) -> Option<F> {
            let mut eval = F::ZERO;
            for (coeff, term) in self.terms().iter() {
                eval += term.evaluate(&point) * coeff;
            }
            Some(eval)
        }
        fn fix_variables(&self, partial_point: &[F]) -> Self {
            let mut res: multivariate::SparsePolynomial<F, SparseTerm> = Self::zero();
            let num_vars: usize = DenseMVPolynomial::num_vars(self);
            let mut full_point: Vec<F> = partial_point.to_vec();
            full_point.append(&mut vec![F::one(); num_vars - partial_point.len()]);

            for (coeff, term) in self.terms() {
                let mut eval: F = term.evaluate(&full_point);
                eval *= coeff;
                let new_term: SparseTerm = SparseTerm::new(
                    term.iter()
                        .filter(|(var, _)| *var >= partial_point.len())
                        .map(|(var, power)| (var - partial_point.len(), *power))
                        .collect(),
                );
                let poly: multivariate::SparsePolynomial<F, SparseTerm> =
                    multivariate::SparsePolynomial {
                        num_vars: num_vars - partial_point.len(),
                        terms: vec![(eval, new_term)],
                    };

                res += &poly;
            }

            res
        }
        fn to_univariate(&self) -> univariate::SparsePolynomial<F> {
            let mut res: univariate::SparsePolynomial<F> = univariate::SparsePolynomial::zero();

            for p in Hypercube::<F>::new(DenseMVPolynomial::num_vars(self) - 1) {
                let mut point: Vec<F> = vec![F::one()];
                point.extend_from_slice(&p);
                let mut r: univariate::SparsePolynomial<F> = univariate::SparsePolynomial::zero();

                for (coeff, term) in self.terms() {
                    let mut eval: F = term.evaluate(&point);
                    let power: usize = term
                        .iter()
                        .find(|(variable, _power)| *variable == 0)
                        .map(|(_variable, power)| *power)
                        .unwrap_or(0);

                    eval *= coeff;

                    r += &univariate::SparsePolynomial::from_coefficients_slice(&[(power, eval)]);
                }
                res += &r;
            }

            res
        }
        fn num_vars(&self) -> usize {
            DenseMVPolynomial::num_vars(self)
        }
        fn to_evaluations(&self) -> Vec<F> {
            Hypercube::<F>::new(DenseMVPolynomial::<F>::num_vars(self))
                .map(|point: Vec<F>| {
                    SumcheckMultivariatePolynomial::<F>::evaluate(self, &point).unwrap()
                })
                .collect()
        }
    }

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]

    pub struct FrConfig;
    pub type TestField = Fp64<MontBackend<FrConfig, 1>>;
    pub type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

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

    pub fn run_boolean_sumcheck_test<F: Field + std::convert::From<i32>, P: Prover<F>>(
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
        let g_round_0 = prover.next_message(None).unwrap();
        assert_eq!(
            g_round_0.evaluate(&F::ZERO),
            F::from(14),
            "g0 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_0.evaluate(&F::ONE),
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
        let g_round_1 = prover.next_message(Some(F::ONE)).unwrap(); // x0 fixed to one
        assert_eq!(
            g_round_0.evaluate(&F::ONE),
            g_round_1.evaluate(&F::ZERO) + g_round_1.evaluate(&F::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&F::ZERO),
            F::from(4),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&F::ONE),
            F::from(7),
            "g1 should evaluate correctly for input 1"
        );
        // LAST ROUND x1 fixed to 1
        // 110 = 0
        // sum g(0) = 0
        // 111 = 7
        // sum g(1) = 7
        let g_round_2 = prover.next_message(Some(F::ONE)).unwrap(); // x1 fixed to one
        assert_eq!(
            g_round_1.evaluate(&F::ONE),
            g_round_2.evaluate(&F::ZERO) + g_round_2.evaluate(&F::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&F::ZERO),
            F::from(0),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&F::ONE),
            F::from(7),
            "g2 should evaluate correctly for input 1"
        );
    }

    pub fn run_basic_sumcheck_test<F: Field + std::convert::From<i32>, P: Prover<F>>(
        mut prover: P,
    ) {
        // FIRST ROUND x0 fixed to 3
        // 3,0,1 = 6
        // 3,0,0 = 6
        // sum g1(0) = 12
        // 3,1,1 = 38 = 0 mod 19
        // 3,1,0 = 31 = 12 mod 19
        // sum g1(1) = 12
        let g_round_0 = prover.next_message(None).unwrap();
        let g_round_1 = prover.next_message(Some(F::from(3))).unwrap(); // x0 fixed to 3
        assert_eq!(
            g_round_0.evaluate(&F::from(3)),
            g_round_1.evaluate(&F::ZERO) + g_round_1.evaluate(&F::ONE)
        );
        assert_eq!(
            g_round_1.evaluate(&F::ZERO),
            F::from(12),
            "g1 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_1.evaluate(&F::ONE),
            F::from(12),
            "g1 should evaluate correctly for input 1"
        );
        // LAST ROUND x1 fixed to 4
        // 3,4,0 = 108 = 11 mod 19
        // sum g(0) = 11
        // 3,4,1 = 138 = 1 mod 19
        // sum g(1) = 1
        let g_round_2 = prover.next_message(Some(F::from(4))).unwrap(); // x1 fixed to 4
        assert_eq!(
            g_round_1.evaluate(&F::from(4)),
            g_round_2.evaluate(&F::ZERO) + g_round_2.evaluate(&F::ONE)
        );
        assert_eq!(
            g_round_2.evaluate(&F::ZERO),
            F::from(11),
            "g2 should evaluate correctly for input 0"
        );
        assert_eq!(
            g_round_2.evaluate(&F::ONE),
            F::from(1),
            "g2 should evaluate correctly for input 1"
        );
    }
}
