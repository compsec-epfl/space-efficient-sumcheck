use num_bigint::BigUint;

use crate::fields::m31::{Field32, FIELD_32_MODULUS};

impl From<Field32> for BigUint {
    fn from(field: Field32) -> BigUint {
        BigUint::from(field.value)
    }
}

impl From<bool> for Field32 {
    fn from(b: bool) -> Self {
        Field32 {
            value: if b { 1 } else { 0 },
        }
    }
}

impl From<u8> for Field32 {
    fn from(u: u8) -> Self {
        Field32 { value: u as u32 }
    }
}

impl From<u16> for Field32 {
    fn from(u: u16) -> Self {
        Field32 { value: u as u32 }
    }
}

impl From<u32> for Field32 {
    fn from(u: u32) -> Self {
        let value = if u == FIELD_32_MODULUS {
            0
        } else if u > FIELD_32_MODULUS {
            u % FIELD_32_MODULUS
        } else {
            u
        };
        Field32 { value }
    }
}

impl From<u64> for Field32 {
    fn from(u: u64) -> Self {
        let value = if u == FIELD_32_MODULUS as u64 {
            0
        } else if u > FIELD_32_MODULUS as u64 {
            (u % FIELD_32_MODULUS as u64) as u32
        } else {
            u as u32
        };
        Field32 { value }
    }
}

impl From<u128> for Field32 {
    fn from(u: u128) -> Self {
        let value = if u == FIELD_32_MODULUS as u128 {
            0
        } else if u > FIELD_32_MODULUS as u128 {
            (u % FIELD_32_MODULUS as u128) as u32
        } else {
            u as u32
        };
        Field32 { value }
    }
}
