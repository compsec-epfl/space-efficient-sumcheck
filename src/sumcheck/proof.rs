use ark_ff::Field;
use ark_poly::{univariate::SparsePolynomial, Polynomial};
use ark_std::{rand::Rng, vec::Vec};

use crate::sumcheck::Prover;

#[derive(Debug)]
pub struct Sumcheck<F: Field> {
    pub prover_messages: Vec<SparsePolynomial<F>>,
    pub verifier_messages: Vec<F>,
    pub is_accepted: bool,
}

impl<F: Field> Sumcheck<F> {
    pub fn prove<P: Prover<F>, R: Rng>(mut prover: P, rng: &mut R) -> Self {
        let mut prover_messages: Vec<SparsePolynomial<F>> =
            Vec::with_capacity(prover.total_rounds());
        let mut verifier_messages: Vec<F> = Vec::with_capacity(prover.total_rounds());
        let mut is_accepted = true;

        // run the protocol
        let mut verifier_message: Option<F> = None;
        while let Some(message) = prover.next_message(verifier_message) {
            // perfom checks
            let round_evaluation: F = message.evaluate(&F::ZERO) + message.evaluate(&F::ONE);
            let is_round_accepted: bool;
            if round_evaluation == prover.claimed_evaluation() {
                is_round_accepted = round_evaluation == prover.claimed_evaluation();
            } else {
                // only add verifier message to transcript when not "None"
                verifier_messages.push(verifier_message.unwrap());
                is_round_accepted = round_evaluation
                    == prover_messages
                        .last()
                        .unwrap()
                        .evaluate(&verifier_message.unwrap());
            }

            // always add prover message to transcript
            prover_messages.push(message);

            // if round not accepted terminate the protocol
            if !is_round_accepted {
                is_accepted = false;
                break;
            }

            // othewise sample a random field element and proceed
            verifier_message = Some(F::rand(rng));
        }

        // done.
        Sumcheck {
            prover_messages,
            verifier_messages,
            is_accepted,
        }
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
    };

    use crate::sumcheck::BasicProver;

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
    fn basic() {
        let prover = BasicProver::<TestField, TestPolynomial>::new(test_polynomial());
        let rng = &mut ark_std::test_rng();
        let transcript = Sumcheck::<TestField>::prove(prover, rng);
        assert_eq!(transcript.is_accepted, true);
    }
}
