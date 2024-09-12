use num_bigint::BigUint;

use crate::fields::m31::{M31, M31_MODULUS};

impl From<M31> for BigUint {
    fn from(field: M31) -> BigUint {
        BigUint::from(field.value)
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
    fn from(u: u8) -> Self {
        M31 { value: u as u32 }
    }
}

impl From<u16> for M31 {
    fn from(u: u16) -> Self {
        M31 { value: u as u32 }
    }
}

impl From<u32> for M31 {
    fn from(u: u32) -> Self {
        let value = if u == M31_MODULUS {
            0
        } else if u > M31_MODULUS {
            u % M31_MODULUS
        } else {
            u
        };
        M31 { value }
    }
}

impl From<u64> for M31 {
    fn from(u: u64) -> Self {
        let value = if u == M31_MODULUS as u64 {
            0
        } else if u > M31_MODULUS as u64 {
            (u % M31_MODULUS as u64) as u32
        } else {
            u as u32
        };
        M31 { value }
    }
}

impl From<u128> for M31 {
    fn from(u: u128) -> Self {
        let value = if u == M31_MODULUS as u128 {
            0
        } else if u > M31_MODULUS as u128 {
            (u % M31_MODULUS as u128) as u32
        } else {
            u as u32
        };
        M31 { value }
    }
}
