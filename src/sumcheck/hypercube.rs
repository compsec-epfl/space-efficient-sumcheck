use std::marker::PhantomData;

use ark_ff::Field;

pub struct BooleanHypercube<F: Field> {
    n: usize,
    current: usize,
    __f: PhantomData<F>,
}

impl<F: Field> BooleanHypercube<F> {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            current: 0,
            __f: PhantomData,
        }
    }
}

impl<F: Field> Iterator for BooleanHypercube<F> {
    type Item = Vec<F>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 2usize.pow(self.n as u32) {
            return None;
        }
    
        let point_binary_str = format!("{:0width$b}", self.current, width = self.n as usize);
        let point: Vec<F> = point_binary_str.chars().map(|c| if c == '0' { F::ZERO } else { F::ONE }).collect();
    
        self.current += 1;
        Some(point)
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
        let hypercube = BooleanHypercube::<TestField>::new(0);
        let points = vec![vec![TestField::ZERO], vec![TestField::ONE]];
        for (i, point) in hypercube.enumerate() {
            assert_eq!(points[i], point);
        }
    }

    #[test]
    fn numerical_order() {
        let hypercube = BooleanHypercube::<TestField>::new(3);
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
