use ark_ff::{BigInt, BigInteger256};
use ark_std::{num::ParseIntError, str::FromStr};
use num_bigint::BigUint;

use crate::fields::m31::{
    M31, M31_MODULUS_I32, M31_MODULUS_U128, M31_MODULUS_U32, M31_MODULUS_U64, M31_MODULUS_USIZE,
};

impl From<M31> for BigInt<4> {
    fn from(field: M31) -> BigInt<4> {
        BigInt::<4>([field.value as u64, 0, 0, 0])
    }
}

impl From<BigUint> for M31 {
    fn from(biguint: BigUint) -> Self {
        let reduced_value = biguint % BigUint::from(M31_MODULUS_U32);
        let value = reduced_value.to_u32_digits().get(0).copied().unwrap_or(0);
        M31::from(value)
    }
}

impl From<BigInteger256> for M31 {
    fn from(bigint: BigInteger256) -> Self {
        let bigint_u64 = bigint.0[0];
        let reduced_value = bigint_u64 % (M31_MODULUS_U64);
        let value = reduced_value as u32;
        M31::from(value)
    }
}

impl FromStr for M31 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = usize::from_str(s)?;
        let reduced_value = value % M31_MODULUS_USIZE;
        Ok(M31::from(reduced_value as u32))
    }
}

impl From<M31> for BigUint {
    fn from(element: M31) -> BigUint {
        BigUint::from(element.value)
    }
}

impl From<bool> for M31 {
    fn from(b: bool) -> Self {
        M31 {
            value: if b { 1 } else { 0 },
        }
    }
}

impl From<u8> for M31 {
    fn from(value: u8) -> Self {
        M31 {
            value: value as u32,
        }
    }
}

impl From<u16> for M31 {
    fn from(value: u16) -> Self {
        M31 {
            value: value as u32,
        }
    }
}

impl From<u32> for M31 {
    fn from(value: u32) -> Self {
        M31 {
            value: if value == M31_MODULUS_U32 {
                0
            } else if value > M31_MODULUS_U32 {
                value % M31_MODULUS_U32
            } else {
                value
            },
        }
    }
}

impl From<i32> for M31 {
    fn from(value: i32) -> Self {
        M31 {
            value: if value == M31_MODULUS_I32 {
                0
            } else if value < 0 {
                (M31_MODULUS_I32 - value) as u32
            } else {
                value as u32
            },
        }
    }
}

impl From<u64> for M31 {
    fn from(value: u64) -> Self {
        M31 {
            value: if value == M31_MODULUS_U64 {
                0
            } else if value > M31_MODULUS_U64 {
                (value % M31_MODULUS_U64) as u32 // TODO: replace
            } else {
                value as u32
            },
        }
    }
}

impl From<u128> for M31 {
    fn from(value: u128) -> Self {
        M31 {
            value: if value == M31_MODULUS_U128 {
                0
            } else if value > M31_MODULUS_U128 {
                (value % M31_MODULUS_U128) as u32 // TODO: replace
            } else {
                value as u32
            },
        }
    }
}
