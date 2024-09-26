use ark_ff::{FftField, Field, One, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use zeroize::Zeroize;

use std::simd::cmp::SimdPartialOrd;
use std::simd::{u32x64, Simd};
use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
};

mod field;
pub mod ops;
mod prime_field;
pub mod transmute;

pub const BB_MODULUS_U32: u32 = 0x78000001;
pub const BB_MODULUS_U64: u64 = BB_MODULUS_U32 as u64;
pub const BB_MODULUS_U128: u128 = BB_MODULUS_U32 as u128;
pub const BB_MODULUS_USIZE: usize = BB_MODULUS_U32 as usize;
pub const BB_MODULUS_I32: i32 = BB_MODULUS_U32 as i32;
pub const BB_MODULUS_I64: i64 = BB_MODULUS_U32 as i64;

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
pub struct BabyBear {
    mod_value: u32,
}

const LANES: usize = 64;

impl BabyBear {
    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = self.clone();
        for _ in 0..power_log {
            res = res.square();
        }
        res
    }
    pub fn reduce_sum(vec: &[u32]) -> Self {
        let sum: u32 = vec.iter().fold(0, |acc, &x| {
            let tmp = acc + x;
            if tmp < BB_MODULUS_U32 {
                return tmp;
            } else {
                return tmp - BB_MODULUS_U32;
            }
        });
        Self { mod_value: sum }
    }
    pub fn reduce_sum_packed(values: &[u32]) -> Self {
        assert!(values.len() % LANES == 0);
        let packed_modulus: Simd<u32, LANES> = u32x64::splat(BB_MODULUS_U32);
        let mut packed_sums: Simd<u32, LANES> = u32x64::splat(0);
        for i in (0..values.len()).step_by(64) {
            let tmp_packed_sums = packed_sums + u32x64::from_slice(&values[i..]);
            let is_mod_needed = tmp_packed_sums.simd_ge(packed_modulus);
            packed_sums = is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
        }
        Self::reduce_sum(&packed_sums.to_array())
    }

    fn convert_u32_to_u64(slice: &[u32]) -> Vec<u64> {
        slice.iter().map(|&num| num as u64).collect()
    }
    // pub fn batch_mult_packed(values: &mut [u32]) -> Self {
    //     assert!(values.len() % LANES == 0);
    //     let packed_modulus: Simd<u32, LANES> = u32x64::splat(BB_MODULUS_U32);
    //     for i in (0..values.len()).step_by(64) {
    //         let slice = Self::convert_u32_to_u64(&values[i..]).as_slice();
    //         let tmp_packed_sums = packed_sums + u64x64::from_slice(slice);
    //         let is_mod_needed = tmp_packed_sums.simd_ge(packed_modulus);
    //         packed_sums = is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
    //     }
    //     Self::reduce_sum(&packed_sums.to_array())
    // }
}

impl Zero for BabyBear {
    fn zero() -> Self {
        BabyBear::from(0_u8)
    }
    fn is_zero(&self) -> bool {
        self.mod_value == 0
    }
}

impl One for BabyBear {
    fn one() -> Self {
        BabyBear::from(1_u8)
    }
    fn is_one(&self) -> bool {
        self.to_u32() == 1_u32
    }
}

impl Zeroize for BabyBear {
    fn zeroize(&mut self) {
        todo!()
    }
}

impl Distribution<BabyBear> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BabyBear {
        let value = rng.gen_range(0..BB_MODULUS_U32);
        BabyBear::from(value)
    }
}

impl Display for BabyBear {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.mod_value, f)
    }
}

impl FftField for BabyBear {
    const GENERATOR: Self = BabyBear { mod_value: 5 };

    const TWO_ADICITY: u32 = 1;

    const TWO_ADIC_ROOT_OF_UNITY: Self = BabyBear { mod_value: 5 };

    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = None;

    fn get_root_of_unity(_n: u64) -> Option<Self> {
        None
    }
}

impl CanonicalDeserializeWithFlags for BabyBear {
    #[inline]
    fn deserialize_with_flags<R: Read, F: Flags>(
        _reader: R,
    ) -> Result<(Self, F), SerializationError> {
        Ok((Self { mod_value: 1 }, F::from_u8(1).unwrap()))
    }
}

impl CanonicalSerializeWithFlags for BabyBear {
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

impl Default for BabyBear {
    fn default() -> Self {
        BabyBear::from(1_u8)
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::{Field, UniformRand};
    use ark_std::test_rng;

    use crate::fields::baby_bear::{BabyBear, BB_MODULUS_U32};

    pub fn reduce_sum_sanity(vec: &[u32]) -> BabyBear {
        BabyBear::from(vec.iter().fold(0, |acc, &x| (acc + x) % BB_MODULUS_U32))
    }

    #[test]
    fn inverse() {
        let a = BabyBear::from(2);
        assert_eq!(BabyBear::from(1006632961), a.inverse().unwrap());
    }

    #[test]
    fn reduce_sum_correctness() {
        let random_values: Vec<u32> = (0..2_i32.pow(13))
            .map(|_| BabyBear::rand(&mut test_rng()).to_u32())
            .collect();
        let exp = reduce_sum_sanity(&random_values);
        assert_eq!(exp, BabyBear::reduce_sum(&random_values));
        assert_eq!(exp, BabyBear::reduce_sum_packed(&random_values));
    }
}