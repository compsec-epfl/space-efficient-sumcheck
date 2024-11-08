use ark_ff::{Field, One, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use zeroize::Zeroize;

use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
};

pub mod fft_field;
pub mod field;
pub mod ops;
pub mod prime_field;
pub mod transmute;
pub mod vec_ops_field;

// Mersenne prime 31
pub const M31_MODULUS: u32 = 2147483647;

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord,
    Hash,
    CanonicalDeserialize,
    CanonicalSerialize,
)]

pub struct M31 {
    value: u32,
}

impl M31 {
    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = self.clone();
        for _ in 0..power_log {
            res = res.square();
        }
        res
    }
    pub fn rand(rng: &mut impl Rng) -> Self {
        let value = rng.gen_range(0..M31_MODULUS);
        M31 { value }
    }
}

impl Zero for M31 {
    fn zero() -> Self {
        M31::from(0)
    }
    fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl One for M31 {
    fn one() -> Self {
        M31::from(1)
    }
    fn is_one(&self) -> bool {
        self.value == 1
    }
}

impl Zeroize for M31 {
    fn zeroize(&mut self) {
        todo!()
    }
}

impl Distribution<M31> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> M31 {
        let value = rng.gen_range(0..M31_MODULUS as u64);
        M31::from(value)
    }
}

impl Display for M31 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl CanonicalDeserializeWithFlags for M31 {
    #[inline]
    fn deserialize_with_flags<R: Read, F: Flags>(
        _reader: R,
    ) -> Result<(Self, F), SerializationError> {
        Ok((Self { value: 1 }, F::from_u8(1).unwrap()))
    }
}

impl CanonicalSerializeWithFlags for M31 {
    #[inline]
    fn serialize_with_flags<W: Write, F: Flags>(
        &self,
        _writer: W,
        _flags: F,
    ) -> Result<(), SerializationError> {
        Ok(())
    }

    #[inline]
    fn serialized_size_with_flags<F: Flags>(&self) -> usize {
        1
    }
}

impl Default for M31 {
    fn default() -> Self {
        M31::from(1_u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::{
        m31::{M31, M31_MODULUS},
        VecOpsField,
    };
    use ark_ff::{Field, One, UniformRand, Zero};
    use ark_std::{rand::Rng, test_rng};

    #[test]
    fn is_5_a_generator() {
        fn mod_exp(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
            if modulus == 1 {
                return 0;
            }
            let mut result = 1;
            base = base % modulus;
            while exp > 0 {
                if exp % 2 == 1 {
                    result = (result * base) % modulus;
                }
                exp = exp >> 1;
                base = (base * base) % modulus;
            }
            result
        }
        for i in (M31_MODULUS - 3)..=M31_MODULUS {
            if mod_exp(i as u64, (2) as u64, M31_MODULUS as u64) == 1
                && mod_exp(i as u64, (1) as u64, M31_MODULUS as u64) != 1
            {
                println!("{} is two adic root of unity", i);
            }
        }
        // assert_eq!(mod_exp(7 as u64, (2) as u64, M31_MODULUS_U64), 1);
        // assert_ne!(mod_exp(7 as u64, (2) as u64, M31_MODULUS_U64), 1);
        // let p_minus_one = M31_MODULUS - 1;
        // let factors_of_p_minus_one = vec![2, 3, 7, 11, 31, 151, 331];
        // for d in &factors_of_p_minus_one {
        //     assert_eq!(mod_exp(7 as u64, (2) as u64, M31_MODULUS_U64), 1);
        //     assert_ne!(mod_exp(7 as u64, (2) as u64, M31_MODULUS_U64), 1);
        // }
    }

    #[test]
    fn inverse_correctness() {
        let a = M31::from(2);
        assert_eq!(M31::from(1073741824), a.inverse().unwrap());
    }

    #[test]
    fn reduce_sum_correctness() {
        fn reduce_sum_sanity(vec: &[M31]) -> M31 {
            M31::from(vec.iter().fold(M31::zero(), |acc, &x| (acc + x)))
        }

        let mut rng = test_rng();
        let random_field_values: Vec<M31> = (0..1 << 13).map(|_| M31::rand(&mut rng)).collect();
        let exp = reduce_sum_sanity(&random_field_values);
        assert_eq!(
            exp,
            M31::reduce_sum(&random_field_values) // M31::reduce_sum(unsafe {
                                                  //     from_raw_parts(
                                                  //         random_field_values.as_ptr() as *mut u32,
                                                  //         random_field_values.len(),
                                                  //     )
                                                  // })
        );
    }

    #[test]
    fn batch_mult_correctness() {
        fn test_field_values(mut rng: &mut impl Rng) -> (Vec<M31>, Vec<M31>) {
            let mut exp: Vec<M31> = (0..(1 << 10)).map(|_| M31::rand(&mut rng)).collect();
            exp.push(M31::from(M31_MODULUS - 1));
            exp.push(M31::from(M31_MODULUS - 2));
            exp.push(M31::zero());
            exp.push(M31::one());
            (exp.clone(), exp)
        }
        fn scalar_mult_sanity(values: &mut [M31], scalar: M31) {
            for elem in values.iter_mut() {
                *elem = *elem * scalar;
            }
        }

        let mut rng = test_rng();
        let (mut exp, mut rec) = test_field_values(&mut rng);
        for _ in 0..(1) {
            // get a random scalar
            let scalar = M31::rand(&mut rng);
            println!("scalar is: {}", scalar);
            // apply the scaling
            scalar_mult_sanity(&mut exp, scalar);
            M31::scalar_mult(&mut rec, scalar);
            // check parity
            assert_eq!(exp, rec);
        }
    }
}
