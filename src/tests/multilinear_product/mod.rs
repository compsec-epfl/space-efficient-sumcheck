use crate::{multilinear_product::Prover, streams::EvaluationStream};
use ark_ff::Field;

pub fn sanity_test_4_variables<'a, F: Field, S: EvaluationStream<F>, P: Prover<'a, F, S>>(
    mut prover: P,
) {
    // ZEROTH ROUND
    // 0000 = 0 * 0 = 0
    // 0001 = 1 * 1 = 1
    // 0010 = 0 * 0 = 0
    // 0011 = 1 * 1 = 1
    // 0100 = 13 * 13 = 17
    // 0101 = 14 * 14 = 6
    // 0110 = 1 * 1 = 1
    // 0111 = 2 * 2 = 4
    // sum g0(0) = 11
    // 1000 = 2 * 2 = 4
    // 1001 = 3 * 3 = 9
    // 1010 = 2 * 2 = 4
    // 1011 = 3 * 3 = 9
    // 1100 = 0 * 0 = 0
    // 1101 = 1 * 1 = 1
    // 1110 = 7 * 7 = 11
    // 1111 = 8 * 8 = 7
    // sum g0(1) = 7
    let round_0 = prover.next_message(None).unwrap();
    assert_eq!(
        round_0.0,
        F::from(11_u32),
        "g0 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_0.1,
        F::from(7_u32),
        "g0 should evaluate correctly for input 1"
    );
    // FIRST ROUND x0 fixed to 3
    // 3000 = (0 * 17) + (2 * 3) = 6 * 6 = 17
    // 3001 = (1 * 17) + (3 * 3) = 7 * 7 = 11
    // 3010 = (0 * 17) + (2 * 3) = 6 * 6 = 17
    // 3011 = (1 * 17) + (3 * 3) = 7 * 7 = 11
    // sum g1(0) = 18
    // 3100 = (13 * 17) + (0 * 3) = 12 * 12 = 11
    // 3101 = (14 * 17) + (1 * 3) = 13 * 13 = 17
    // 3110 = (1 * 17) + (7 * 3) = 0 * 0 = 0
    // 3111 = (2 * 17) + (8 * 3) = 1 * 1 = 1
    // sum g1(1) = 10
    let round_1 = prover.next_message(Some(F::from(3_u32))).unwrap(); // x0 fixed to 3
                                                                      // assert_eq!(
                                                                      //     round_0.0 - (round_0.0 - round_0.1) * F::from(3) * F::from(3),
                                                                      //     round_1.0 + round_1.1
                                                                      // );
    assert_eq!(
        round_1.0,
        F::from(18_u32),
        "g1 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_1.1,
        F::from(10_u32),
        "g1 should evaluate correctly for input 1"
    );
    // SECOND ROUND x1 fixed to 4
    // 3400 = (6 * 16) + (12 * 4) = 11 * 11 = 7
    // 3401 = (7 * 16) + (13 * 4) = 12 * 12 = 11
    // sum g2(0) = 18
    // 3410 = (6 * 16) + (0 * 4) = 1 * 1 = 1
    // 3411 = (7 * 16) + (1 * 4) = 2 * 2 = 4
    // sum g2(1) = 5
    let round_2 = prover.next_message(Some(F::from(4_u32))).unwrap(); // x1 fixed to 4
                                                                      // assert_eq!(
                                                                      //     round_1.0 - (round_1.0 - round_1.1) * F::from(4),
                                                                      //     round_2.0 + round_2.1
                                                                      // );
    assert_eq!(
        round_2.0,
        F::from(18_u32),
        "g2 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(5_u32),
        "g2 should evaluate correctly for input 1"
    );
    // LAST ROUND x2 fixed to 7
    // 3470 = (11 * 13) + (1 * 7) = 17 * 17 = 4
    // sum g3(0) = 4
    // 3471 = (12 * 13) + (2 * 7) = 18 * 18 = 1
    // sum g3(1) =
    let round_2 = prover.next_message(Some(F::from(7_u32))).unwrap(); // x2 fixed to 7
                                                                      // assert_eq!(
                                                                      //     round_1.0 - (round_1.0 - round_1.1) * F::from(4),
                                                                      //     round_2.0 + round_2.1
                                                                      // );
    assert_eq!(
        round_2.0,
        F::from(4_u32),
        "g3 should evaluate correctly for input 0"
    );
    assert_eq!(
        round_2.1,
        F::from(1_u32),
        "g3 should evaluate correctly for input 1"
    );
}
