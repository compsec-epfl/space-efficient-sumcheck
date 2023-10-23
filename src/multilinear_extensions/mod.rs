use ark_ff::Field;
// use rayon::{
//     iter::{IndexedParallelIterator, ParallelIterator},
//     prelude::IntoParallelIterator,
// };

// https://github.com/montekki/thaler-study/blob/master/multilinear-extensions/src/lib.rs
// ~ 100ns
// FUNCTION lagrange_basis_poly_at(x: ARRAY of F, w: ARRAY of F) -> OPTIONAL F:
//     IF LENGTH(x) ≠ LENGTH(w) THEN
//         RETURN None
//     END IF

//     // Initialize accumulator to 1
//     accumulator := 1

//     // Iterate through the x and w arrays
//     FOR i FROM 0 TO LENGTH(x) - 1 DO
//         x_i := x[i]
//         w_i := w[i]

//         // Calculate basis term for current pair
//         basis_term := (x_i * w_i) + ((1 - x_i) * (1 - w_i))

//         // Update accumulator
//         accumulator := accumulator * basis_term
//     END FOR

//     // Return the Lagrange basis polynomial wrapped in Some
//     RETURN Some(accumulator)
// END FUNCTION
pub fn lagrange_basis_poly_at<F: Field>(x: &[F], w: &[F]) -> Option<F> {
    if x.len() != w.len() {
        None
    } else {
        Some(
            x.to_vec()
                .iter()
                .zip(w.iter())
                .fold(F::ONE, |acc, (&x_i, &w_i)| {
                    acc * (x_i * w_i + (F::ONE - x_i) * (F::ONE - w_i))
                }),
        )
    }
}

// Evaluate multilinear extension of with an algorithm from CTY11
// ~280µs for 15 variables
// FUNCTION cty_interpolation(evals: ARRAY of F, r: ARRAY of F) -> F:
//     // Initialize the result variable with the additive identity of the field.
//     res := F::ZERO

//     // Iterate over evaluations and construct the interpolated output
//     FOR i FROM 0 TO LENGTH(evals) - 1 DO
//         // Initialize the weight vector with zeros.
//         w := NEW ARRAY of F with size LENGTH(r)

//         // Construct the weight vector based on the current index i.
//         FOR j FROM LENGTH(r) - 1 DOWNTO 0 DO
//             // Calculate the bitmask for the j-th bit.
//             bitmask := 2^j

//             // Check if the j-th bit of the index i is set.
//             IF (i AND bitmask) != 0 THEN
//                 w[j] := F::ONE
//             ELSE
//                 w[j] := F::ZERO
//             END IF
//         END FOR

//         // Compute the Lagrange basis polynomial using the weight vector w and input vector r.
//         basis_poly := lagrange_basis_poly_at(r, w)

//         // Multiply the current evaluation by the computed Lagrange basis polynomial and add it to the result.
//         res := res + evals[i] * basis_poly
//     END FOR

//     // Return the final computed interpolated value.
//     RETURN res
// END FUNCTION
pub fn cty_interpolation<F: Field>(evals: &[F], r: &[F]) -> F {
    // evals
    //     .into_par_iter()
    //     .enumerate()
    //     .map(|(i, &eval)| -> F {
    //         let mut weight: F = F::ONE; // Initialize the weight to 1.

    //         for (j, &r_j) in r.iter().rev().enumerate() {
    //             // Check if the j-th bit of the index i is set.
    //             if (i >> j) & 1 == 1 {
    //                 weight *= r_j;
    //             } else {
    //                 weight *= F::ONE - r_j;
    //             }
    //         }

    //         // Multiply the evaluation by the computed weight and add it to the result.
    //         return eval * weight;
    //     })
    //     .collect::<Vec<F>>()
    //     .iter()
    //     .fold(F::ZERO, |acc, &x| acc + x)
    let mut res: F = F::zero();

    for (i, eval) in evals.iter().enumerate() {
        let mut weight: F = F::one(); // Initialize the weight to 1.

        for (j, &r_j) in r.iter().rev().enumerate() {
            // Check if the j-th bit of the index i is set.
            if (i >> j) & 1 == 1 {
                weight *= r_j;
            } else {
                weight *= F::one() - r_j;
            }
        }

        // Multiply the evaluation by the computed weight and add it to the result.
        res += *eval * weight;
    }

    res
}

// Evaluate multilinear extension with an algorith from [`VSBW13`] Time efficient
// ~110µs for 15 variables
// FUNCTION vsbw_interpolation(evals: ARRAY of F, r: ARRAY of F) -> F:
//     // Initialize the evaluation table with a single element F::one()
//     eval_table := [F::one()]

//     // Iterate over each element in r
//     FOR EACH r_j IN r DO
//         // Create a new empty evaluation table
//         eval_table_new := []

//         // Iterate over each element in the current evaluation table
//         FOR EACH eval IN eval_table DO
//             // Compute two new values by multiplying with (F::one() - r_j) and r_j
//             value1 := eval * (F::one() - r_j)
//             value2 := eval * r_j

//             // Add the computed values to the new evaluation table
//             eval_table_new.push(value1)
//             eval_table_new.push(value2)
//         END FOR

//         // Update the current evaluation table with the new values
//         eval_table := eval_table_new
//     END FOR

//     // Initialize the result to zero
//     result := F::zero()

//     // Iterate over the computed evaluation table and input evaluations
//     FOR i FROM 0 TO LENGTH(eval_table) - 1 DO
//         // Multiply the corresponding evaluation from eval_table with the corresponding input evaluation
//         product := eval_table[i] * evals[i]

//         // Add the product to the result
//         result := result + product
//     END FOR

//     // Return the final computed result
//     RETURN result
// END FUNCTION
pub fn vsbw_interpolation<F: Field>(evals: &[F], r: &[F]) -> F {
    // Initialize the evaluation table with a single element.
    let mut eval_table = vec![F::one()];

    // Temporary vector to store new values without reallocations.
    let mut eval_table_new = Vec::with_capacity(2 * evals.len());

    for &r_j in r {
        // Clear the temporary vector.
        eval_table_new.clear();

        // Extend the temporary vector's capacity.
        eval_table_new.reserve(2 * eval_table.len());

        // Iterate over existing elements in the evaluation table and update them.
        for &eval in &eval_table {
            eval_table_new.push(eval * (F::one() - r_j));
            eval_table_new.push(eval * r_j);
        }

        // Swap the temporary vector with the original evaluation table.
        std::mem::swap(&mut eval_table, &mut eval_table_new);
    }

    // Calculate the final interpolated value using the evaluation table and input evaluations.
    eval_table
        .into_iter()
        .zip(evals.iter())
        .fold(F::zero(), |acc, (w_j, p_j)| acc + w_j * p_j)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{Fp64, MontBackend, MontConfig, PrimeField};

    #[derive(MontConfig)]
    #[modulus = "5"]
    #[generator = "2"]
    struct FrConfig;

    type Fr = Fp64<MontBackend<FrConfig, 1>>;

    #[test]
    fn basic_tests() {
        let evals: Vec<Fr> = [1u32, 2, 1, 4]
            .iter()
            .map(|&f| Fr::from_bigint(f.into()).unwrap())
            .collect();

        let expected_result = vec![
            vec![1, 2, 3, 4, 0],
            vec![1, 4, 2, 0, 3],
            vec![1, 1, 1, 1, 1],
            vec![1, 3, 0, 2, 4],
            vec![1, 0, 4, 3, 2],
        ];

        for i in 0u32..5 {
            let mut line = Vec::with_capacity(5);
            for j in 0u32..5 {
                let f_r = cty_interpolation(
                    &evals,
                    &[
                        Fr::from_bigint(i.into()).unwrap(),
                        Fr::from_bigint(j.into()).unwrap(),
                    ],
                );
                line.push(f_r.into_bigint().as_ref()[0]);
            }
            assert_eq!(line, expected_result[i as usize], "at line {i}");
        }

        for i in 0u32..5 {
            let mut line = Vec::with_capacity(5);
            for j in 0u32..5 {
                let f_r = vsbw_interpolation(
                    &evals,
                    &[
                        Fr::from_bigint(i.into()).unwrap(),
                        Fr::from_bigint(j.into()).unwrap(),
                    ],
                );
                line.push(f_r.into_bigint().as_ref()[0]);
            }
            assert_eq!(line, expected_result[i as usize]);
        }
    }
}
