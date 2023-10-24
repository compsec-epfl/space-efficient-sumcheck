use ark_ff::Field;

pub fn lagrange_polynomial<F: Field>(x: &[F], w: &[F]) -> Option<F> {
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
