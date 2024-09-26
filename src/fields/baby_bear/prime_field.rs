use ark_ff::{BigInt, BigInteger256, PrimeField};

use crate::fields::baby_bear::{BabyBear, BB_MODULUS_U64};

const BB_MODULUS_BIGINT4: BigInt<4> = BigInt::new([BB_MODULUS_U64, 0, 0, 0]);
const BB_MODULUS_MINUS_ONE_DIV_TWO_BIGINT4: BigInt<4> =
    BigInt::new([(BB_MODULUS_U64 - 1) / 2, 0, 0, 0]);

impl PrimeField for BabyBear {
    type BigInt = BigInteger256;

    const MODULUS: Self::BigInt = BB_MODULUS_BIGINT4;

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BB_MODULUS_MINUS_ONE_DIV_TWO_BIGINT4;

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
        Self { mod_value: 0 }
    }

    fn from_le_bytes_mod_order(_bytes: &[u8]) -> Self {
        Self { mod_value: 0 }
    }
}
