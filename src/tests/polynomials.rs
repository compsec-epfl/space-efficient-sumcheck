use crate::hypercube::{Hypercube, HypercubeMember};
use ark_ff::Field;
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    DenseMVPolynomial,
};

/*
 * These are two small polynomials to use for sanity checking.
 */

pub fn three_variable_polynomial<F: Field>() -> Vec<F> {
    // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2
    return multivariate::SparsePolynomial::from_coefficients_slice(
        3,
        &[
            (
                F::from(4_u32),
                multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
            ),
            (
                F::from(7_u32),
                multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
            ),
            (F::from(2_u32), multivariate::SparseTerm::new(vec![(0, 1)])),
            (F::from(13_u32), multivariate::SparseTerm::new(vec![(1, 1)])),
        ],
    )
    .to_evaluations();
}

pub fn four_variable_polynomial<F: Field>() -> Vec<F> {
    // 4*x_1*x_2 + 7*x_2*x_3 + 2*x_1 + 13*x_2 + 1x_4
    return multivariate::SparsePolynomial::from_coefficients_slice(
        4,
        &[
            (
                F::from(4_u32),
                multivariate::SparseTerm::new(vec![(0, 1), (1, 1)]),
            ),
            (
                F::from(7_u32),
                multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
            ),
            (F::from(2_u32), multivariate::SparseTerm::new(vec![(0, 1)])),
            (F::from(13_u32), multivariate::SparseTerm::new(vec![(1, 1)])),
            (F::from(1_u32), multivariate::SparseTerm::new(vec![(3, 1)])),
        ],
    )
    .to_evaluations();
}

/*
 * Below here we extend multivariate::SparsePolynomial<F, SparseTerm> so that we can
 * get evaluations over the boolean hypercube
 *
 * The idea comes from here: https://github.com/montekki/thaler-study/blob/master/sum-check-protocol/src/lib.rs
 */

pub trait Polynomial<F: Field> {
    // Evaluates the polynomial at the provided point (expressed as a hypercube member)
    // using the given number of variables.
    fn evaluate(&self, num_vars: usize, point: HypercubeMember) -> Option<F>;

    // Converts the polynomial into a vector containing evaluations at every
    // point of the hypercube.
    fn to_evaluations(&self) -> Vec<F>;
}

impl<F: Field> Polynomial<F> for multivariate::SparsePolynomial<F, SparseTerm> {
    fn evaluate(&self, num_vars: usize, point: HypercubeMember) -> Option<F> {
        // Convert the boolean representation into field elements.
        let mut field_values: Vec<F> = Vec::with_capacity(num_vars);
        for bit in point {
            field_values.push(if bit { F::ONE } else { F::ZERO });
        }

        // Compute the evaluation by summing the contributions of each term.
        let mut result = F::ZERO;
        for (coefficient, term) in self.terms().iter() {
            result += term.evaluate(&field_values) * coefficient;
        }

        Some(result)
    }

    fn to_evaluations(&self) -> Vec<F> {
        let num_vars = DenseMVPolynomial::<F>::num_vars(self);
        let total_points = Hypercube::stop_value(num_vars);
        let mut evaluations = Vec::with_capacity(total_points);

        // Iterate through each index of the hypercube.
        for index in 0..total_points {
            let point = HypercubeMember::new(num_vars, index);
            let value = Self::evaluate(self, num_vars, point).unwrap();
            evaluations.push(value);
        }

        evaluations
    }
}
