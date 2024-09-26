use ark_ff::{BigInt, BigInteger256};
use ark_std::{num::ParseIntError, str::FromStr};
use num_bigint::BigUint;
use num_traits::{PrimInt, Unsigned};

use crate::fields::baby_bear::{
    BabyBear, BB_MODULUS_I32, BB_MODULUS_U128, BB_MODULUS_U32, BB_MODULUS_U64, BB_MODULUS_USIZE,
};

fn mod_transmute_signed<S, T>(source: S, modulus: S) -> T
where
    S: PrimInt,
    T: PrimInt,
{
    T::from(if source >= modulus {
        source - modulus
    } else if source < S::zero() {
        source + modulus
    } else {
        source
    })
    .unwrap()
}

pub fn mod_transmute_unsigned<S, T>(source: S, modulus: S) -> T
where
    S: PrimInt + Unsigned,
    T: PrimInt + Unsigned,
{
    T::from(match source >= modulus {
        true => source % modulus,
        false => source,
    })
    .unwrap()
}

impl BabyBear {
    pub fn to_u32(&self) -> u32 {
        self.mod_value
    }
}

impl BabyBear {
    pub fn to_u64(&self) -> u64 {
        self.mod_value as u64
    }
}

impl From<BabyBear> for BigInt<4> {
    fn from(source: BabyBear) -> BigInt<4> {
        BigInt::<4>([source.to_u64(), 0, 0, 0])
    }
}

impl From<BigUint> for BabyBear {
    fn from(source: BigUint) -> Self {
        let reduced_value = source % BigUint::from(BB_MODULUS_U32);
        let value = reduced_value.to_u32_digits().get(0).copied().unwrap_or(0);
        BabyBear::from(value)
    }
}

impl From<BigInteger256> for BabyBear {
    fn from(source: BigInteger256) -> Self {
        // TODO: fix this
        let bigint_u64 = source.0[0];
        let reduced_value = bigint_u64 % (BB_MODULUS_U64);
        let value = reduced_value as u32;
        BabyBear::from(value)
    }
}

impl FromStr for BabyBear {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: fix this
        let value = usize::from_str(s)?;
        let reduced_value = value % BB_MODULUS_USIZE;
        Ok(BabyBear::from(reduced_value as u32))
    }
}

impl From<BabyBear> for BigUint {
    fn from(source: BabyBear) -> BigUint {
        BigUint::from(source.to_u32())
    }
}

impl From<bool> for BabyBear {
    fn from(source: bool) -> Self {
        BabyBear {
            mod_value: match source {
                true => 1,
                false => 0,
            },
        }
    }
}

impl From<i32> for BabyBear {
    fn from(source: i32) -> Self {
        BabyBear {
            mod_value: mod_transmute_signed::<i32, u32>(source, BB_MODULUS_I32),
        }
    }
}

impl From<u8> for BabyBear {
    fn from(source: u8) -> Self {
        BabyBear {
            mod_value: source as u32,
        }
    }
}

impl From<u16> for BabyBear {
    fn from(source: u16) -> Self {
        BabyBear {
            mod_value: source as u32,
        }
    }
}

impl From<u32> for BabyBear {
    fn from(source: u32) -> Self {
        BabyBear {
            mod_value: mod_transmute_unsigned::<u32, u32>(source, BB_MODULUS_U32),
        }
    }
}

impl From<u64> for BabyBear {
    fn from(source: u64) -> Self {
        BabyBear {
            mod_value: mod_transmute_unsigned::<u64, u32>(source, BB_MODULUS_U64),
        }
    }
}

impl From<u128> for BabyBear {
    fn from(source: u128) -> Self {
        BabyBear {
            mod_value: mod_transmute_unsigned::<u128, u32>(source, BB_MODULUS_U128),
        }
    }
}

mod tests {
    use crate::fields::baby_bear::BabyBear;

    #[test]
    fn from_bool() {
        assert_eq!(BabyBear::from(1_u32), BabyBear::from(true));
        assert_eq!(BabyBear::from(0_u32), BabyBear::from(false));
    }

    #[test]
    fn from_i32() {
        assert_eq!(BabyBear::from(127_u32), BabyBear::from(127_i32));
        assert_eq!(
            BabyBear::from(2147483647_u32),
            BabyBear::from(2147483647_i32)
        );
        assert_eq!(BabyBear::from(2013265794), BabyBear::from(-127_i32));
    }

    #[test]
    fn from_u8() {
        assert_eq!(BabyBear::from(127_u32), BabyBear::from(127_u8));
        assert_eq!(BabyBear::from(255_u32), BabyBear::from(255_u8));
    }

    #[test]
    fn from_u16() {
        assert_eq!(BabyBear::from(127_u32), BabyBear::from(127_u16));
        assert_eq!(BabyBear::from(65535_u32), BabyBear::from(65535_u16));
    }

    #[test]
    fn from_u32() {
        assert_eq!(BabyBear::from(127_u64), BabyBear::from(127_u32));
        assert_eq!(
            BabyBear::from(268435453_u32),
            BabyBear::from(4294967295_u32)
        );
    }

    #[test]
    fn from_u64() {
        assert_eq!(BabyBear::from(127_u32), BabyBear::from(127_u64));
        assert_eq!(
            BabyBear::from(1641596511_u32),
            BabyBear::from(9999999999999999999_u64)
        );
    }

    #[test]
    fn from_u128() {
        assert_eq!(BabyBear::from(127_u32), BabyBear::from(127_u128));
        assert_eq!(
            BabyBear::from(1570367446_u32),
            BabyBear::from(9999999999999999999999999999999999_u128)
        );
    }
}
