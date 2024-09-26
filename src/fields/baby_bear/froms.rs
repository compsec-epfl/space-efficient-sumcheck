use ark_ff::{BigInt, BigInteger256};
use ark_std::{num::ParseIntError, str::FromStr};
use num_bigint::BigUint;

use crate::fields::baby_bear::{
    BabyBear, BB_MODULUS_I32, BB_MODULUS_U128, BB_MODULUS_U32, BB_MODULUS_U64, BB_MODULUS_USIZE,
};

impl BabyBear {
    pub fn to_u64(&self) -> u64 {
        self.value as u64
    }
}

impl From<BabyBear> for BigInt<4> {
    fn from(field: BabyBear) -> BigInt<4> {
        BigInt::<4>([field.value as u64, 0, 0, 0])
    }
}

impl From<BigUint> for BabyBear {
    fn from(biguint: BigUint) -> Self {
        let reduced_value = biguint % BigUint::from(BB_MODULUS_U32);
        let value = reduced_value.to_u32_digits().get(0).copied().unwrap_or(0);
        BabyBear::from(value)
    }
}

impl From<BigInteger256> for BabyBear {
    fn from(bigint: BigInteger256) -> Self {
        let bigint_u64 = bigint.0[0];
        let reduced_value = bigint_u64 % (BB_MODULUS_U64);
        let value = reduced_value as u32;
        BabyBear::from(value)
    }
}

impl FromStr for BabyBear {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = usize::from_str(s)?;
        let reduced_value = value % BB_MODULUS_USIZE;
        Ok(BabyBear::from(reduced_value as u32))
    }
}

impl From<BabyBear> for BigUint {
    fn from(element: BabyBear) -> BigUint {
        BigUint::from(element.value)
    }
}

impl From<bool> for BabyBear {
    fn from(b: bool) -> Self {
        BabyBear {
            value: if b { 1 } else { 0 },
        }
    }
}

impl From<u8> for BabyBear {
    fn from(value: u8) -> Self {
        BabyBear {
            value: value as u32,
        }
    }
}

impl From<u16> for BabyBear {
    fn from(value: u16) -> Self {
        BabyBear {
            value: value as u32,
        }
    }
}

impl From<u32> for BabyBear {
    fn from(value: u32) -> Self {
        BabyBear {
            value: if value == BB_MODULUS_U32 {
                0
            } else if value > BB_MODULUS_U32 {
                value % BB_MODULUS_U32
            } else {
                value
            },
        }
    }
}

impl From<i32> for BabyBear {
    fn from(value: i32) -> Self {
        BabyBear {
            value: if value == BB_MODULUS_I32 {
                0
            } else if value < 0 {
                (BB_MODULUS_I32 - value) as u32
            } else {
                value as u32
            },
        }
    }
}

impl From<u64> for BabyBear {
    fn from(value: u64) -> Self {
        BabyBear {
            value: if value == BB_MODULUS_U64 {
                0
            } else if value > BB_MODULUS_U64 {
                (value % BB_MODULUS_U64) as u32 // TODO: replace
            } else {
                value as u32
            },
        }
    }
}

impl From<u128> for BabyBear {
    fn from(value: u128) -> Self {
        BabyBear {
            value: if value == BB_MODULUS_U128 {
                0
            } else if value > BB_MODULUS_U128 {
                (value % BB_MODULUS_U128) as u32 // TODO: replace
            } else {
                value as u32
            },
        }
    }
}
