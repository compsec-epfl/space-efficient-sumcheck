use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};

use crate::{hypercube::Hypercube, messages::VerifierMessages, tests::polynomials::Polynomial};
pub struct BasicProductProver<F: Field> {
    pub claim: F,
    pub current_round: usize,
    pub inverse_four: F,
    pub num_variables: usize,
    pub p: SparsePolynomial<F, SparseTerm>,
    pub q: SparsePolynomial<F, SparseTerm>,
    pub verifier_messages: VerifierMessages<F>,
}

impl<F: Field> BasicProductProver<F> {
    pub fn compute_round(&self) -> (F, F, F) {
        let mut sum0 = F::ZERO;
        let mut sum1 = F::ZERO;
        for (_, b) in Hypercube::new(self.num_variables - self.current_round) {
            let mut partial_point: Vec<F> = b
                .to_vec_bool()
                .into_iter()
                .map(|bit: bool| -> F {
                    if bit {
                        F::ONE
                    } else {
                        F::ZERO
                    }
                })
                .collect();
            let update_zero = partial_point[0] == F::ZERO;
            let mut point: Vec<F> = self.verifier_messages.messages.clone();
            point.append(&mut partial_point);

            let p_val = self.p.evaluate(point.clone()).unwrap();
            let q_val = self.q.evaluate(point).unwrap();
            let val = p_val * q_val;
            if update_zero {
                sum0 += val;
            } else {
                sum1 += val;
            }
        }
        // TODO(z-tech): sum_half
        (sum0, sum1, F::ONE)
    }
    pub fn is_initial_round(&self) -> bool {
        self.current_round == 0
    }
    pub fn total_rounds(&self) -> usize {
        self.num_variables
    }
}
