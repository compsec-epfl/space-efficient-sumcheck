use ark_ff::biginteger::BigInteger256;
use ark_ff::{BigInt, FftField, Field, One, PrimeField, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use num_bigint::BigUint;
use zeroize::Zeroize;

use std::simd::u64x4;
use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
    num::ParseIntError,
    str::FromStr,
};

pub mod froms;
pub mod ops;

/// Mersenne prime 31
pub const M31_MODULUS: u32 = (1 << 31) - 1;

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
    pub fn reduce_sum(vec: Vec<Self>) -> Self {
        let mut sums = u64x4::from_array([
            vec[0].value as u64,
            vec[1].value as u64,
            vec[2].value as u64,
            vec[3].value as u64,
        ]);
        let modulus = u64x4::from_array([M31_MODULUS as u64; 4]);

        for (i, chunk) in vec.chunks(4).enumerate() {
            if i == 0 {
                continue;
            }

            let next_4: [u64; 4] = match chunk.len() {
                1 => [chunk[0].value as u64, 0, 0, 0],
                2 => [chunk[0].value as u64, chunk[1].value as u64, 0, 0],
                3 => [
                    chunk[0].value as u64,
                    chunk[1].value as u64,
                    chunk[2].value as u64,
                    0,
                ],
                4 => [
                    chunk[0].value as u64,
                    chunk[1].value as u64,
                    chunk[2].value as u64,
                    chunk[3].value as u64,
                ],
                _ => todo!(),
            };

            sums = sums + u64x4::from_array(next_4);
            sums = sums % modulus;
        }

        let sum: usize = (sums[0] + sums[1] + sums[2] + sums[3]).try_into().unwrap();
        let sum = sum % M31_MODULUS as usize;

        Self { value: sum as u32 }
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
        // Overwrite the sensitive fields with zero
        // self.value.zeroize();
    }
}

impl Distribution<M31> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> M31 {
        let value = rng.gen_range(0..M31_MODULUS);
        M31::from(value)
    }
}

impl From<M31> for BigInt<4> {
    fn from(field: M31) -> BigInt<4> {
        BigInt::<4>([field.value as u64, 0, 0, 0])
    }
}

impl From<BigUint> for M31 {
    fn from(biguint: BigUint) -> Self {
        let reduced_value = biguint % BigUint::from(M31_MODULUS);
        let value = reduced_value.to_u32_digits().get(0).copied().unwrap_or(0);
        M31::from(value)
    }
}

impl From<BigInteger256> for M31 {
    fn from(bigint: BigInteger256) -> Self {
        let bigint_u64 = bigint.0[0];
        let reduced_value = bigint_u64 % (M31_MODULUS as u64);
        let value = reduced_value as u32;
        M31::from(value)
    }
}

impl FromStr for M31 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = usize::from_str(s)?;
        let reduced_value = value % M31_MODULUS as usize;
        Ok(M31::from(reduced_value as u32))
    }
}

impl Display for M31 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl PrimeField for M31 {
    type BigInt = BigInteger256;

    // TODO: fix this
    const MODULUS: Self::BigInt = BigInteger256::one(); // ark_ff::BigInt::<4>::from(M31_MODULUS);

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger256::one();

    const MODULUS_BIT_SIZE: u32 = 31;

    const TRACE: Self::BigInt = BigInteger256::one();

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
        M31::from((2 * self.value) % M31_MODULUS)
    }

    fn inverse(&self) -> Option<Self> {
        if self.value == 0 {
            return None;
        }
        Some(Self::from((1 / self.value) % M31_MODULUS))
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
        todo!()
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
    fn accumulate() {
        let v = vec![
            M31::from(0_u32),
            M31::from(1_u32),
            M31::from(2_u32),
            M31::from(3_u32),
            M31::from(4_u32),
            M31::from(5_u32),
            M31::from(6_u32),
            M31::from(7_u32),
            M31::from(8_u32),
        ];
        let accumulated = M31::reduce_sum(v);
        assert_eq!(accumulated, M31::from(36_u32))
    }
}
