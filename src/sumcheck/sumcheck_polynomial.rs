use ark_ff::{Field, Zero};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    polynomial::DenseMVPolynomial,
    univariate, Polynomial,
};

use crate::sumcheck::BooleanHypercube;

/// https://github.com/montekki/thaler-study/blob/master/sum-check-protocol/src/lib.rs

/// An abstraction over all types of polynomials that may be
/// used in a sumcheck protocol.
pub trait SumcheckPolynomial<F: Field> {
    /// Evaluates `self` at a given point
    ///
    /// Return `None` if dimentionality of `point` does not match
    /// an expected one.
    fn evaluate(&self, point: &[F]) -> Option<F>;

    /// Reduce the number of variables in `Self` by fixing a
    /// `partial_point.len()` variables at `partial_point`.
    fn fix_variables(&self, partial_point: &[F]) -> Self;

    /// Compute the $j$-th round of polynomial for sumcheck over
    /// first variable.
    ///
    /// Reduces to univariate polynomial of first variable:
    ///
    /// $$
    /// \sum_{(x_{j+1},\cdots,x_{\nu}) \in \lbrace 0, 1 \rbrace ^{\nu - 1}}
    /// g(X_j,x_{j+1},x_{j+2},\cdots,x_{\nu})
    /// $$
    ///
    /// Note that the initial polynomial $g(x_1,\cdots,x_{\nu})$ that
    /// the protocol was started with is supposed to become
    /// $g(r_1,r_2,\cdots,x_j,\cdots,x_n)$ at this point by calling [`fix_variables`]
    ///
    ///
    /// [`fix_variables`]: SumcheckPolynomial::fix_variables
    fn to_univariate(&self) -> univariate::SparsePolynomial<F>;

    /// Returns the number of variables in `self`
    fn num_vars(&self) -> usize;

    /// Returns a list of evaluations over the domain, which is the
    /// boolean hypercube.
    fn to_evaluations(&self) -> Vec<F>;
}

impl<F: Field> SumcheckPolynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, point: &[F]) -> Option<F> {
        Some(Polynomial::evaluate(self, &point.to_owned()))
    }

    fn fix_variables(&self, partial_point: &[F]) -> Self {
        let mut res = Self::zero();
        let num_vars = DenseMVPolynomial::num_vars(self);
        let mut full_point = partial_point.to_vec();
        full_point.append(&mut vec![F::one(); num_vars - partial_point.len()]);

        for (coeff, term) in self.terms() {
            let mut eval = term.evaluate(&full_point);
            eval *= coeff;
            let new_term = SparseTerm::new(
                term.iter()
                    .filter(|(var, _)| *var >= partial_point.len())
                    .map(|(var, power)| (var - partial_point.len(), *power))
                    .collect(),
            );
            let poly = multivariate::SparsePolynomial {
                num_vars: num_vars - partial_point.len(),
                terms: vec![(eval, new_term)],
            };

            res += &poly;
        }

        res
    }

    fn to_univariate(&self) -> univariate::SparsePolynomial<F> {
        let mut res = univariate::SparsePolynomial::zero();

        for p in BooleanHypercube::<F>::new((DenseMVPolynomial::num_vars(self) - 1) as u32) {
            let mut point = vec![F::one()];
            point.extend_from_slice(&p);
            let mut r = univariate::SparsePolynomial::zero();

            for (coeff, term) in self.terms() {
                let mut eval = term.evaluate(&point);
                let power = term
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
        BooleanHypercube::new(DenseMVPolynomial::num_vars(self) as u32)
            .map(|point| Polynomial::evaluate(self, &point))
            .collect()
    }
}