use std::marker::PhantomData;

use ark_ff::Field;

pub struct BooleanHypercube<F: Field> {
    n: u32,
    current: u64,
    __f: PhantomData<F>,
}

impl<F: Field> BooleanHypercube<F> {
    pub fn new(n: u32) -> Self {
        Self {
            n,
            current: 0,
            __f: PhantomData,
        }
    }
}

impl<F: Field> Iterator for BooleanHypercube<F> {
    type Item = Vec<F>;

    // TODO: (z-tech) improve this
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 2u64.pow(self.n) {
            None
        } else {
            let mut point_binary_str: String = format!("{:b}", self.current);
            if self.n > 1 {
                let zero_padding: String = vec!['0'; (self.n as usize) - point_binary_str.len()]
                    .into_iter()
                    .collect();
                point_binary_str = zero_padding + &point_binary_str;
            }

            let mut point = Vec::<F>::with_capacity(self.n as usize);
            for bit in point_binary_str.chars() {
                if bit == '0' {
                    point.push(F::ZERO);
                } else {
                    point.push(F::ONE);
                }
            }
            self.current += 1;
            Some(point)
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

    use pretty_assertions::assert_eq;

    #[derive(MontConfig)]
    #[modulus = "19"]
    #[generator = "2"]
    struct FrConfig;

    type TestField = Fp64<MontBackend<FrConfig, 1>>;

    #[test]
    fn small_n() {
        let hypercube = BooleanHypercube::<TestField>::new(0_u32);
        let points = vec![vec![TestField::ZERO], vec![TestField::ONE]];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }

    #[test]
    fn numerical_order() {
        let hypercube = BooleanHypercube::<TestField>::new(3_u32);
        let points = vec![
            vec![TestField::ZERO, TestField::ZERO, TestField::ZERO],
            vec![TestField::ZERO, TestField::ZERO, TestField::ONE],
            vec![TestField::ZERO, TestField::ONE, TestField::ZERO],
            vec![TestField::ZERO, TestField::ONE, TestField::ONE],
            vec![TestField::ONE, TestField::ZERO, TestField::ZERO],
            vec![TestField::ONE, TestField::ZERO, TestField::ONE],
            vec![TestField::ONE, TestField::ONE, TestField::ZERO],
            vec![TestField::ONE, TestField::ONE, TestField::ONE],
        ];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }
}
