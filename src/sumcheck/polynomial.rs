use ark_ff::{Field, Zero};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    polynomial::DenseMVPolynomial,
    univariate, Polynomial,
};

use crate::sumcheck::BooleanHypercube;

/// https://github.com/montekki/thaler-study/blob/master/sum-check-protocol/src/lib.rs

pub trait SumcheckMultivariatePolynomial<F: Field> {
    fn evaluate(&self, point: &[F]) -> Option<F>;
    fn fix_variables(&self, partial_point: &[F]) -> Self;
    fn to_univariate(&self) -> univariate::SparsePolynomial<F>;
    fn num_vars(&self) -> usize;
    fn to_evaluations(&self) -> Vec<F>;
}

impl<F: Field> SumcheckMultivariatePolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, point: &[F]) -> Option<F> {
        Some(Polynomial::evaluate(self, &point.to_owned()))
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
            let poly: multivariate::SparsePolynomial<F, SparseTerm> = multivariate::SparsePolynomial {
                num_vars: num_vars - partial_point.len(),
                terms: vec![(eval, new_term)],
            };

            res += &poly;
        }

        res
    }
    fn to_univariate(&self) -> univariate::SparsePolynomial<F> {
        let mut res: univariate::SparsePolynomial<F> = univariate::SparsePolynomial::zero();

        for p in BooleanHypercube::<F>::new((DenseMVPolynomial::num_vars(self) - 1) as u32) {
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
        BooleanHypercube::<F>::new(DenseMVPolynomial::<F>::num_vars(self) as u32)
            .map(|point: Vec<F>| Polynomial::<F>::evaluate(self, &point))
            .collect()
    }
}