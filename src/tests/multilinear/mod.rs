use ark_ff::Field;

use crate::{
    multilinear::{Prover, ProverConfig},
    streams::EvaluationStream,
    tests::{polynomials::three_variable_polynomial, streams::BasicEvaluationStream},
};

pub fn multilinear_round_sanity<F, S, P>(p: &mut P, message: Option<F>, eval_0: F, eval_1: F)
where
    F: Field,
    S: EvaluationStream<F>,
    P: Prover<F, VerifierMessage = Option<F>, ProverMessage = Option<(F, F)>>,
{
    let round = p.next_message(message).unwrap();
    assert_eq!(round.0, eval_0, "g0 should evaluate correctly",);
    assert_eq!(round.1, eval_1, "g1 should evaluate correctly",);
}

pub fn sanity_test<F, S, P>()
where
    F: Field,
    S: EvaluationStream<F> + From<BasicEvaluationStream<F>>,
    P: Prover<F, VerifierMessage = Option<F>, ProverMessage = Option<(F, F)>>,
    P::ProverConfig: ProverConfig<F, S>,
{
    let s: S = BasicEvaluationStream::new(three_variable_polynomial()).into();
    let mut p = P::new(P::ProverConfig::default(F::from(6_u32), 3, s));
    /*
     * Zeroth Round: All variables are free
     *
     * Evaluations at different input points:
     *   (0,0,0) →  0
     *   (0,0,1) →  0
     *   (0,1,0) → 13
     *   (0,1,1) →  1
     *   -----------------
     *   Sum g₀(0) ≡ 14
     *
     *   (1,0,0) →  2
     *   (1,0,1) →  2
     *   (1,1,0) →  0
     *   (1,1,1) →  7
     *   -----------------
     *   Sum g₀(1) ≡ 11
     */
    multilinear_round_sanity::<F, S, P>(&mut p, None, F::from(14_u32), F::from(11_u32));
    /*
     * First Round: x₀ fixed to 3
     *
     * Evaluations at different input points:
     *   (3,0,1) →  6
     *   (3,0,0) →  6
     *   -----------------
     *   Sum g₁(0) ≡ 12
     *
     *   (3,1,1) → 38 ≡  0 (mod 19)
     *   (3,1,0) → 31 ≡ 12 (mod 19)
     *   -----------------
     *   Sum g₁(1) ≡ 12
     */
    multilinear_round_sanity::<F, S, P>(
        &mut p,
        Some(F::from(3_u32)),
        F::from(12_u32),
        F::from(12_u32),
    );
    /*
     * Last Round: x₁ fixed to 4
     *
     * Evaluations at different input points:
     *   (3,4,0) → 108 ≡ 11 (mod 19)
     *   -----------------
     *   Sum g(0) ≡ 11
     *
     *   (3,4,1) → 134 ≡  1 (mod 19)
     *   -----------------
     *   Sum g(1) ≡ 1
     */
    multilinear_round_sanity::<F, S, P>(
        &mut p,
        Some(F::from(4_u32)),
        F::from(11_u32),
        F::from(1_u32),
    );
}

// pub fn sanity_test_boolean_only<'a, F: Field, S: EvaluationStream<F>, P: Prover<'a, F, S>>(
//     mut prover: P,
// ) {
//     // ZEROTH ROUND
//     // all variables free
//     // 000 = 0
//     // 001 = 0
//     // 010 = 13
//     // 011 = 1
//     // sum g0(0) = 14
//     // 100 = 2
//     // 110 = 0
//     // 101 = 2
//     // 111 = 7
//     // sum g0(1) = 11
//     let round_0 = prover.next_message(None).unwrap();
//     assert_eq!(
//         round_0.0,
//         F::from(14_u32),
//         "g0 should evaluate correctly for input 0"
//     );
//     assert_eq!(
//         round_0.1,
//         F::from(11_u32),
//         "g0 should evaluate correctly for input 1"
//     );
//     // FIRST ROUND x0 fixed to 1
//     // 101 = 2
//     // 100 = 2
//     // sum g1(0) = 4
//     // 111 = 7
//     // 110 = 0
//     // sum g1(1) = 7
//     let round_1 = prover.next_message(Some(F::ONE)).unwrap(); // x0 fixed to one
//     assert_eq!(round_0.1, round_1.0 + round_1.1);
//     assert_eq!(
//         round_1.0,
//         F::from(4_u32),
//         "g1 should evaluate correctly for input 0"
//     );
//     assert_eq!(
//         round_1.1,
//         F::from(7_u32),
//         "g1 should evaluate correctly for input 1"
//     );
//     // LAST ROUND x1 fixed to 1
//     // 110 = 0
//     // sum g(0) = 0
//     // 111 = 7
//     // sum g(1) = 7
//     let round_2 = prover.next_message(Some(F::ONE)).unwrap(); // x1 fixed to one
//     assert_eq!(round_1.1, round_2.0 + round_2.1);
//     assert_eq!(
//         round_2.0,
//         F::from(0_u32),
//         "g2 should evaluate correctly for input 0"
//     );
//     assert_eq!(
//         round_2.1,
//         F::from(7_u32),
//         "g2 should evaluate correctly for input 1"
//     );
// }

pub fn sanity_test_3_variables<F, S, P>(mut prover: P)
where
    F: Field,
    S: EvaluationStream<F>,
    P: Prover<F, VerifierMessage = Option<F>, ProverMessage = Option<(F, F)>>,
{
    // let mut p0 = P::new(P::ProverArgs());
    // FIRST ROUND x0 fixed to 3
    // 3,0,1 = 6
    // 3,0,0 = 6
    // sum g1(0) = 12
    // 3,1,1 = 38 = 0 mod 19
    // 3,1,0 = 31 = 12 mod 19
    // sum g1(1) = 12
    let round_0 = prover.next_message(None).unwrap();
    let round_1 = prover.next_message(Some(F::from(3_u32))).unwrap(); // x0 fixed to 3
    assert_eq!(
        round_0.0 - (round_0.0 - round_0.1) * F::from(3_u32),
        round_1.0 + round_1.1
    );
    assert_eq!(
        round_1.0,
        F::from(12_u32),
        "g1 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_1.1,
        F::from(12_u32),
        "g1 should evaluate correctly for input 1"
    );
    // LAST ROUND x1 fixed to 4
    // 3,4,0 = 108 = 11 mod 19
    // sum g(0) = 11
    // 3,4,1 = 134 = 1 mod 19
    // sum g(1) = 1
    let round_2 = prover.next_message(Some(F::from(4_u32))).unwrap(); // x1 fixed to 4
    assert_eq!(
        round_1.0 - (round_1.0 - round_1.1) * F::from(4_u32),
        round_2.0 + round_2.1
    );
    assert_eq!(
        round_2.0,
        F::from(11_u32),
        "g2 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(1_u32),
        "g2 should evaluate correctly for input 1"
    );
}
