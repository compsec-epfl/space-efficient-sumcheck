use ark_ff::{BigInt, BigInteger256, PrimeField};

use super::{M31, M31_MODULUS};

pub const M31_MODULUS_BIGINT4: BigInt<4> = BigInt::new([M31_MODULUS as u64, 0, 0, 0]);
pub const M31_MODULUS_MINUS_ONE_DIV_TWO_BIGINT4: BigInt<4> =
    BigInt::new([(M31_MODULUS as u64 - 1) / 2, 0, 0, 0]);

impl PrimeField for M31 {
    type BigInt = BigInteger256;

    const MODULUS: Self::BigInt = M31_MODULUS_BIGINT4;

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = M31_MODULUS_MINUS_ONE_DIV_TWO_BIGINT4;

    const MODULUS_BIT_SIZE: u32 = 32;

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
