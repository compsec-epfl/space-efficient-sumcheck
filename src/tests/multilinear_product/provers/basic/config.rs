use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};

pub struct BasicProductProverConfig<F: Field> {
    pub claim: F,
    pub num_variables: usize,
    pub p: SparsePolynomial<F, SparseTerm>,
    pub q: SparsePolynomial<F, SparseTerm>,
}

impl<F: Field> BasicProductProverConfig<F> {
    pub fn new(
        claim: F,
        num_variables: usize,
        p: SparsePolynomial<F, SparseTerm>,
        q: SparsePolynomial<F, SparseTerm>,
    ) -> Self {
        Self {
            claim,
            num_variables,
            p,
            q,
        }
    }
}
