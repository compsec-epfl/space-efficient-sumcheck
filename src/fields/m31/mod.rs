use ark_ff::biginteger::{BigInt, BigInteger256};
use ark_ff::{FftField, Field, One, PrimeField, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use zeroize::Zeroize;

use std::arch::aarch64::{
    uint32x4_t, vaddq_u32, vbslq_u32, vcgeq_u32, vdupq_n_u32, vld1q_u32, vst1q_u32, vsubq_u32,
};
use std::intrinsics::simd::simd_cast;
use std::simd::{cmp::SimdPartialOrd, u32x64, Simd};
use std::simd::{u64x64, Mask};
use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
};

pub mod fft_field;
pub mod field;
pub mod ops;
pub mod prime_field;
pub mod transmute;

// Mersenne prime 31
pub const M31_MODULUS: u32 = 2147483647;
pub const M31_MODULUS_U32: u32 = (1 << 31) - 1;
pub const M31_MODULUS_I32: i32 = M31_MODULUS_U32 as i32;
pub const M31_MODULUS_U64: u64 = (1 << 31) - 1;
pub const M31_MODULUS_I64: i64 = (1 << 31) - 1;
pub const M31_MODULUS_U128: u128 = (1 << 31) - 1;
pub const M31_MODULUS_USIZE: usize = (1 << 31) - 1;

const LANES: usize = 64;

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
    pub fn batch_mult_normal(values: &mut [u32], multipland: u32) {
        for elem in values.iter_mut() {
            *elem = ((*elem as u64 * multipland as u64) % M31_MODULUS_U64) as u32;
        }
    }

    pub fn batch_mult_trick(values: &mut [u32], multipland: u32) {
        for elem in values.iter_mut() {
            let mut product = *elem as u64 * multipland as u64;
            product = (product & M31_MODULUS_U64) + (product >> 31);
            product = (product & M31_MODULUS_U64) + (product >> 31);
            *elem = product as u32;
        }
    }

    pub fn batch_mult_parts(values: &mut [u32], multiplicand: u32) {
        let multiplicand_lo = multiplicand & 0xFFFF;
        let multiplicand_hi = multiplicand >> 16;
        for elem in values.iter_mut() {
            // split the value
            let lo = *elem & 0xFFFF;
            let hi = *elem >> 16;

            // carry out the multiplication
            let mut hi_hi = hi * multiplicand_hi;
            let mut hi_lo = hi * multiplicand_lo;
            let mut lo_hi = lo * multiplicand_hi;
            let mut lo_lo = lo * multiplicand_lo;

            // reduce into M31
            hi_hi = hi_hi << 1;
            hi_lo = ((hi_lo << 16) & M31_MODULUS) + (hi_lo >> 15);
            lo_hi = ((lo_hi << 16) & M31_MODULUS) + (lo_hi >> 15);
            lo_lo = (lo_lo & M31_MODULUS) + (lo_lo >> 31);

            *elem = Self::reduce_sum(&[hi_hi, hi_lo, lo_hi, lo_lo]).to_u32();
        }
    }

    pub fn batch_sum_packed(values: &mut [u32]) {
        assert!(values.len() % LANES == 0);
        let packed_modulus: Simd<u32, LANES> = u32x64::splat(M31_MODULUS);
        let x = u32x64::splat(9999999);
        // let mut packed_sums: Simd<u32, LANES> = u32x64::splat(0);
        for i in (0..values.len()).step_by(LANES) {
            let mut tmp_packed_sums: Simd<u32, LANES> =
                x + u32x64::from_slice(&values[i..i + LANES]);
            let is_mod_needed: Mask<i32, LANES> = tmp_packed_sums.simd_ge(packed_modulus);
            tmp_packed_sums =
                is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
            unsafe {
                tmp_packed_sums.store_select_ptr(
                    values.as_mut_ptr().wrapping_add(i),
                    Mask::<i32, LANES>::splat(true),
                )
            };
        }
    }

    // pub fn batch_mult_trick_packed(values: &mut [u32], multiplicand: u32) {
    //     assert!(values.len() % LANES == 0);
    //     let multiplicand: Simd<u64, LANES> = u64x64::splat(multiplicand as u64);
    //     let modulus: Simd<u64, LANES> = u64x64::splat(M31_MODULUS_U64);
    //     for i in (0..values.len()).step_by(64) {
    //         // widen
    //         let widened: &[u64] = &values[i..i + 64].to_vec().iter().map(|a| { *a as u64}).collect::<Vec<u64>>();
    //         // multiply
    //         let mut product = u64x64::from_slice(widened) * multiplicand;
    //         // reduce
    //         product = (product & modulus) + (product >> 31);
    //         product = (product & modulus) + (product >> 31);
    //         // narrow
    //         let narrowed: &[u32] = &product.to_array().iter().map(|a| { *a as u32}).collect::<Vec<u32>>();
    //         // write back in
    //         values[i..i + 64].copy_from_slice(&narrowed);
    //     }
    // }

    pub fn batch_mult_trick_packed(values: &mut [u32], multiplicand: u32) {
        assert!(values.len() % LANES == 0);
        let multiplicand: Simd<u64, LANES> = u64x64::splat(multiplicand as u64);
        let modulus: Simd<u64, LANES> = u64x64::splat(M31_MODULUS_U64);
        for i in (0..values.len()).step_by(64) {
            // widen
            let widened: Simd<u64, 64> =
                unsafe { simd_cast(u32x64::from_slice(&values[i..i + 64])) };
            // multiply
            let mut product = widened * multiplicand;
            // reduce
            product = (product & modulus) + (product >> 31);
            product = (product & modulus) + (product >> 31);
            // narrow
            let narrowed: Simd<u32, 64> = unsafe { simd_cast(product) };
            // write back in
            values[i..i + 64].copy_from_slice(&narrowed.to_array());
        }
    }

    pub fn batch_mult_trick_parts_packed(values: &mut [u32], multiplicand: u32) {
        assert!(values.len() % LANES == 0);
        let multiplicand_lo: Simd<u32, LANES> = u32x64::splat(multiplicand & 0xFFFF);
        let multiplicand_hi: Simd<u32, LANES> = u32x64::splat(multiplicand >> 16);
        let modulus: Simd<u32, LANES> = u32x64::splat(M31_MODULUS);
        for i in (0..values.len()).step_by(64) {
            // split the value
            let tmp_values: Simd<u32, 64> = u32x64::from_slice(&values[i..i + 64]);
            let lo = tmp_values & u32x64::splat(0xFFFF);
            let hi = tmp_values >> 16;

            // carry out the multiplication
            let mut hi_hi = hi * multiplicand_hi;
            let mut hi_lo = hi * multiplicand_lo;
            let mut lo_hi = lo * multiplicand_hi;
            let mut lo_lo = lo * multiplicand_lo;

            // reduce into M31
            hi_hi = hi_hi << 1;
            hi_lo = ((hi_lo << 16) & modulus) + (hi_lo >> 15);
            lo_hi = ((lo_hi << 16) & modulus) + (lo_hi >> 15);
            lo_lo = (lo_lo & modulus) + (lo_lo >> 31);

            // combine
            let mut full_product = hi_hi + hi_lo;
            full_product = (full_product & modulus) + (full_product >> 31);
            full_product = full_product + lo_hi;
            full_product = (full_product & modulus) + (full_product >> 31);
            full_product = full_product + lo_lo;
            full_product = (full_product & modulus) + (full_product >> 31);

            // write back in
            values[i..i + 64].copy_from_slice(&full_product.to_array());
        }
    }

    pub fn reduce_sum(vec: &[u32]) -> Self {
        let reduced_sum: u32 = vec.iter().fold(0, |acc, &x| {
            let sum = acc + x;
            if sum < M31_MODULUS {
                return sum;
            } else {
                return sum - M31_MODULUS;
            }
        });
        Self { value: reduced_sum }
    }

    pub fn reduce_sum_packed(values: &[u32]) -> Self {
        assert!(values.len() % LANES == 0);
        let packed_modulus: Simd<u32, LANES> = u32x64::splat(M31_MODULUS);
        let mut packed_sums: Simd<u32, LANES> = u32x64::splat(0);
        for i in (0..values.len()).step_by(LANES) {
            let tmp_packed_sums: Simd<u32, LANES> =
                packed_sums + u32x64::from_slice(&values[i..i + LANES]);
            let is_mod_needed: Mask<i32, LANES> = tmp_packed_sums.simd_ge(packed_modulus);
            packed_sums = is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
        }
        Self::reduce_sum(&packed_sums.to_array())
    }

    /*
        NOTE questions here are:
            (a) How wide is the largest register?
                - SVE (Scalable Vector Extension) support means 256-bits
                - Otherwise 128-bits
            (b) How many of these registers does the system have?
                - ARMv8-A and higher (64-bit ARM processors): 32 registers (v0 to v31).
                - ARMv7-A and lower (32-bit ARM processors): 16 registers (d0 to d15).
    */
    pub fn reduce_sum_packed_neon(values: &[u32]) -> Self {
        assert!(values.len() % LANES == 0);
        let packed_modulus: uint32x4_t = unsafe { vdupq_n_u32(M31_MODULUS) };
        let mut packed_sums: uint32x4_t = unsafe { vdupq_n_u32(0) };
        for i in (0..values.len()).step_by(64) {
            for j in (0..64).step_by(4) {
                let tmp_packed_sums: uint32x4_t =
                    unsafe { vaddq_u32(packed_sums, vld1q_u32(values.as_ptr().add(i + j))) };
                let is_mod_needed: uint32x4_t =
                    unsafe { vcgeq_u32(tmp_packed_sums, packed_modulus) };
                packed_sums = unsafe {
                    vbslq_u32(
                        is_mod_needed,
                        vsubq_u32(tmp_packed_sums, packed_modulus),
                        tmp_packed_sums,
                    )
                };
            }
        }

        // TODO: use all 16 registers at once
        // pub fn reduce_sum_packed_neon(values: &[u32]) -> Self {
        //     assert!(values.len() % LANES == 0);
        //     let packed_modulus: &[uint32x4_t; 16] = &[ unsafe { vdupq_n_u32(M31_MODULUS) }; 16];
        //     let packed_sums: &mut [uint32x4_t; 16] = &mut [ unsafe { vdupq_n_u32(0) }; 16];
        //     for i in (0..values.len()).step_by(64) {
        //         for j in (0..64).step_by(4) {
        //             let tmp_packed_sums: uint32x4_t = unsafe { vaddq_u32(packed_sums[j], vld1q_u32(values.as_ptr().add(i + j))) };
        //             let is_mod_needed: uint32x4_t = unsafe { vcgeq_u32(tmp_packed_sums, packed_modulus[j]) };
        //             packed_sums[j] = unsafe { vbslq_u32(is_mod_needed, vsubq_u32(tmp_packed_sums, packed_modulus[j]), tmp_packed_sums) };
        //         }
        //     }

        //     // Sum up the remaining values in the vector and reduce to a single value
        //     let mut result_outer = [0u32; 64];
        //     for j in (0..64).step_by(4) {
        //         let mut result = [0u32; 4];
        //         unsafe {
        //             vst1q_u32(result.as_mut_ptr(), packed_sums[j]);
        //         }
        //         result_outer[j] = result[0];
        //         result_outer[j+1] = result[1];
        //         result_outer[j+2] = result[2];
        //         result_outer[j+3] = result[3];
        //     }

        //     Self::reduce_sum(&result_outer)
        // }

        // Sum up the remaining values in the vector and reduce to a single value
        let mut result = [0u32; 4];
        unsafe {
            vst1q_u32(result.as_mut_ptr(), packed_sums);
        }

        Self::reduce_sum(&result)
    }

    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = self.clone();
        for _ in 0..power_log {
            res = res.square();
        }
        res
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
        let value = rng.gen_range(0..M31_MODULUS_U32);
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
    use std::simd::{u32x4, u32x64, Simd};

    use ark_ff::{Field, UniformRand};
    use ark_std::test_rng;

    use crate::fields::m31::{M31, M31_MODULUS};

    #[test]
    fn inverse_correctness() {
        let a = M31::from(2);
        assert_eq!(M31::from(1073741824), a.inverse().unwrap());
    }

    #[test]
    fn reduce_sum_correctness() {
        fn reduce_sum_sanity(vec: &[u32]) -> M31 {
            M31::from(vec.iter().fold(0, |acc, &x| (acc + x) % M31_MODULUS))
        }

        let mut rng = test_rng();
        let random_field_values: Vec<u32> =
            (0..1 << 13).map(|_| M31::rand(&mut rng).to_u32()).collect();
        let exp = reduce_sum_sanity(&random_field_values);
        assert_eq!(exp, M31::reduce_sum(&random_field_values));
        assert_eq!(exp, M31::reduce_sum_packed(&random_field_values));
        assert_eq!(exp, M31::reduce_sum_packed_neon(&random_field_values));
    }

    #[test]
    fn batch_mult_correctness() {
        // get some random field values and be sure to add some suspicious ones (make len divisible by LANES=64)
        let mut rng = test_rng();
        let mut exp: Vec<u32> = (0..(1 << 13) - 4)
            .map(|_| M31::rand(&mut rng).to_u32())
            .collect();
        exp.push(M31_MODULUS - 1);
        exp.push(M31_MODULUS - 2);
        exp.push(0);
        exp.push(1);
        let mut act: Vec<u32> = exp.clone();
        for _ in 0..(1 << 8) {
            // try many multiplicands
            let multiplicand = M31::rand(&mut rng).to_u32();
            M31::batch_mult_normal(&mut exp, multiplicand);
            M31::batch_mult_trick_packed(&mut act, multiplicand);
            assert_eq!(exp, act);
        }
        // let multiplicand = 9999999_u32;
        // let mut exp: Vec<u32> = (0..1 << 13).map(|_| M31::rand(&mut rng).to_u32()).collect();
        // exp.push(M31_MODULUS - 1);
        // let mut act: Vec<u32> = exp.clone();
        // M31::batch_mult_normal(&mut exp, multiplicand);
        // M31::batch_mult_parts(&mut act, multiplicand);
        // assert_eq!(exp, act);
    }
}
