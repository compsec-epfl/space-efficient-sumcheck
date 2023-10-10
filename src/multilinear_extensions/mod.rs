use ark_ff::Field;

/// https://github.com/montekki/thaler-study/blob/master/multilinear-extensions/src/lib.rs

fn lagrange_basis_poly_at<F: Field>(x: &[F], w: &[F]) -> Option<F> {
    if x.len() != w.len() {
        None
    } else {
        let res = x.iter().zip(w.iter()).fold(F::one(), |acc, (&x_i, &w_i)| {
            acc * (x_i * w_i + (F::one() - x_i) * (F::one() - w_i))
        });

        Some(res)
    }
}

/// Evaluate multilinear extension of with an algorithm from [`CTY11`]. Space efficient
///
/// [`CTY11`]: https://arxiv.org/abs/1109.6882
pub fn cti_multilinear_from_evaluations<F: Field>(evals: &[F], r: &[F]) -> F {
    let mut res: F = F::zero();

    for (i, eval) in evals.iter().enumerate() {
        let mut w: Vec<F> = Vec::with_capacity(r.len());

        let len: usize = r.len();

        for j in (0..len).rev() {
            let bit: usize = 2_usize.pow(j as u32);

            let w_j: F = if i & bit == 0 { F::zero() } else { F::one() };
            w.push(w_j);
        }

        res += *eval * lagrange_basis_poly_at(r, &w).unwrap();
    }

    res
}


/// Evaluate multilinear extension with an algorith from [`VSBW13`] Time efficient
///
/// [`VSBW13`]: https://ieeexplore.ieee.org/document/6547112
pub fn vsbw_multilinear_from_evaluations<F: Field>(evals: &[F], r: &[F]) -> F {
    let mut eval_table = vec![F::one()];

    for r_j in r {
        let mut eval_table_new = Vec::with_capacity(eval_table.len() * 2);

        for eval in eval_table.into_iter() {
            eval_table_new.push(eval * (F::one() - r_j));
            eval_table_new.push(eval * r_j);
        }

        eval_table = eval_table_new;
    }

    eval_table
        .into_iter()
        .zip(evals.iter())
        .fold(F::zero(), |acc, (w_j, p_j)| acc + w_j * p_j)
}

#[cfg(test)]
mod tests {
    use ark_ff::{Fp64, MontBackend, MontConfig, PrimeField};
    use pretty_assertions::assert_eq;

    use super::*;

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
                let f_r = cti_multilinear_from_evaluations(
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
                let f_r = vsbw_multilinear_from_evaluations(
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