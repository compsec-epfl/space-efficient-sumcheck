use ark_ff::biginteger::{BigInt, BigInteger256};
use ark_ff::{FftField, Field, One, PrimeField, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use zeroize::Zeroize;

use std::simd;
use std::simd::{u64x4, LaneCount};
use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
};

pub mod froms;
pub mod ops;

// Mersenne prime 31
pub const M31_MODULUS_U32: u32 = (1 << 31) - 1;
pub const M31_MODULUS_I32: i32 = M31_MODULUS_U32 as i32;
pub const M31_MODULUS_U64: u64 = (1 << 31) - 1;
pub const M31_MODULUS_I64: i64 = (1 << 31) - 1;
pub const M31_MODULUS_U128: u128 = (1 << 31) - 1;
pub const M31_MODULUS_USIZE: usize = (1 << 31) - 1;
pub const M31_MODULUS_BIGINT4: BigInt<4> = BigInt::new([M31_MODULUS_U64, 0, 0, 0]);

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
    pub fn reduce_sum(vec: &[u64]) -> Self {
        let mut sums = u64x4::from_array([0, 0, 0, 0]);
        let modulus = u64x4::from_array([M31_MODULUS_U64; 4]);
        for chunk in vec.chunks(4) {
            let next_4: [u64; 4] = match chunk.len() {
                1 => [chunk[0], 0, 0, 0],
                2 => [chunk[0], chunk[1], 0, 0],
                3 => [chunk[0], chunk[1], chunk[2], 0],
                4 => [chunk[0], chunk[1], chunk[2], chunk[3]],
                _ => todo!(),
            };

            sums = sums + u64x4::from_array(next_4);
            sums = sums % modulus;
        }

        let mut sum: u64 = (sums[0] + sums[1] + sums[2] + sums[3]).try_into().unwrap();
        sum = sum % M31_MODULUS_U64;

        Self { value: sum as u32 }
    }
    // pub fn reduce_sum_2(data: &[u32]) -> u32 {
    //     // TODO: if we're adding less than 4 billion values < 2^32 can we be sure we won't overflow u64?
    //     // Chunk the data into sections that SIMD can process
    //     let chunk_size = 32;

    //     // Use parallel iterator over chunks
    //     let simd_sum = data
    //         .par_chunks(chunk_size)
    //         .map(|chunk| {
    //             // Load into SIMD registers
    //             let mut simd_chunk = u64x4::splat(0);
    //             for &value in chunk {
    //                 simd_chunk += u64x4::from(value);
    //             }
    //             // Reduce the SIMD vector into a scalar sum
    //             simd_chunk.wrapping_sum()
    //         })
    //         .reduce(|| 0, |acc, x| acc.wrapping_add(x));

    //     // Calculate the final result modulo n
    //     simd_sum % modulo
    // }
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
        M31::from(0_u32)
    }
    fn is_zero(&self) -> bool {
        self.value == 0_u32
    }
}

impl One for M31 {
    fn one() -> Self {
        M31::from(1_u32)
    }
    fn is_one(&self) -> bool {
        self.value == 1_u32
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

impl PrimeField for M31 {
    type BigInt = BigInteger256;

    const MODULUS: Self::BigInt = M31_MODULUS_BIGINT4;

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger256::one();

    const MODULUS_BIT_SIZE: u32 = 31;

    // TODO: what is this?
    const TRACE: Self::BigInt = BigInteger256::one();
    // TODO: what is this?
    const TRACE_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger256::one();

    fn from_bigint(_repr: Self::BigInt) -> Option<Self> {
        todo!()
    }

    fn into_bigint(self) -> Self::BigInt {
        todo!()
    }

    fn from_be_bytes_mod_order(_bytes: &[u8]) -> Self {
        Self { value: 0 }
    }

    fn from_le_bytes_mod_order(_bytes: &[u8]) -> Self {
        Self { value: 0 }
    }
}

impl FftField for M31 {
    const GENERATOR: Self = M31 { value: 5 };

    const TWO_ADICITY: u32 = 1;

    const TWO_ADIC_ROOT_OF_UNITY: Self = M31 { value: 5 };

    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = None;

    fn get_root_of_unity(_n: u64) -> Option<Self> {
        None
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

// Implement the Field trait
impl Field for M31 {
    type BasePrimeField = Self;

    type BasePrimeFieldIter = std::iter::Empty<Self>;

    const SQRT_PRECOMP: Option<ark_ff::SqrtPrecomputation<Self>> = None;

    const ZERO: Self = Self { value: 0 };

    const ONE: Self = Self { value: 1 };

    fn double(&self) -> Self {
        M31::from((2 * self.value) % M31_MODULUS_U32)
    }

    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        let x = *self;
        let y = x.exp_power_of_2(2) * x;
        let z = y.square() * y;
        let a = z.exp_power_of_2(4) * z;
        let b = a.exp_power_of_2(4);
        let c = b * z;
        let d = b.exp_power_of_2(4) * a;
        let e = d.exp_power_of_2(12) * c;
        let f = e.exp_power_of_2(3) * y;
        Some(f)
    }

    fn frobenius_map(&self, _: usize) -> M31 {
        Self { value: self.value }
    }

    fn extension_degree() -> u64 {
        todo!()
    }

    fn to_base_prime_field_elements(&self) -> Self::BasePrimeFieldIter {
        todo!()
    }

    fn from_base_prime_field_elems(_elems: &[Self::BasePrimeField]) -> Option<Self> {
        todo!()
    }

    fn from_base_prime_field(_elem: Self::BasePrimeField) -> Self {
        todo!()
    }

    fn double_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn neg_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn from_random_bytes_with_flags<F: Flags>(_bytes: &[u8]) -> Option<(Self, F)> {
        todo!()
    }

    fn legendre(&self) -> ark_ff::LegendreSymbol {
        todo!()
    }

    fn square(&self) -> Self {
        self.clone() * self.clone()
    }

    fn square_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        todo!()
    }

    fn frobenius_map_in_place(&mut self, _power: usize) {
        todo!()
    }

    fn characteristic() -> &'static [u64] {
        &[]
    }

    fn from_random_bytes(_bytes: &[u8]) -> Option<Self> {
        std::unimplemented!()
    }

    fn sqrt(&self) -> Option<Self> {
        std::unimplemented!()
    }

    fn sqrt_in_place(&mut self) -> Option<&mut Self> {
        std::unimplemented!()
    }

    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }

    fn pow<S: AsRef<[u64]>>(&self, _exp: S) -> Self {
        *self
    }

    fn pow_with_table<S: AsRef<[u64]>>(_powers_of_2: &[Self], _exp: S) -> Option<Self> {
        std::unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::m31::M31;

    #[test]
    fn reduce_sum() {
        assert_eq!(
            M31::reduce_sum(&[0, 1, 2, 3, 4, 5, 6, 7, 8]),
            M31::from(36_u32)
        )
    }
}
