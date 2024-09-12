use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::m31::{Field32, FIELD_32_MODULUS};

impl<'a> DivAssign<&'a mut Field32> for Field32 {
    fn div_assign(&mut self, other: &'a mut Field32) {
        // if other.value == 0 {
        //     panic!("Division by zero");
        // }

        // // Perform division in the field
        // let modulus = self.modulus;
        // let inverse = mod_inverse(other.value, modulus).expect("No modular inverse exists");

        // self.value = (self.value.wrapping_mul(inverse)) % modulus;
    }
}

impl<'a> MulAssign<&'a mut Field32> for Field32 {
    fn mul_assign(&mut self, other: &'a mut Field32) {
        self.value = (self.value.wrapping_mul(other.value)) % FIELD_32_MODULUS;
    }
}

impl<'a> SubAssign<&'a mut Field32> for Field32 {
    fn sub_assign(&mut self, other: &'a mut Field32) {
        let modulus = FIELD_32_MODULUS;
        self.value = (self.value.wrapping_sub(other.value)) % modulus;

        // Handle negative results by adding modulus
        if self.value > modulus {
            self.value += modulus;
        }
    }
}
impl<'a> AddAssign<&'a mut Field32> for Field32 {
    fn add_assign(&mut self, other: &'a mut Field32) {
        self.value = (self.value.wrapping_add(other.value)) % FIELD_32_MODULUS;
    }
}
impl<'a> MulAssign<&'a Field32> for Field32 {
    fn mul_assign(&mut self, other: &'a Field32) {
        self.value = (self.value.wrapping_mul(other.value)) % FIELD_32_MODULUS;
    }
}
impl<'a> SubAssign<&'a Field32> for Field32 {
    fn sub_assign(&mut self, other: &'a Field32) {
        self.value = (self.value.wrapping_sub(other.value)) % FIELD_32_MODULUS;

        // Handle negative results by adding modulus
        if self.value > FIELD_32_MODULUS {
            self.value += FIELD_32_MODULUS;
        }
    }
}

impl<'a> AddAssign<&'a Field32> for Field32 {
    fn add_assign(&mut self, other: &'a Field32) {
        self.value = (self.value.wrapping_add(other.value)) % FIELD_32_MODULUS;
    }
}

// Basic
impl Add for Field32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new((self.value + other.value) % FIELD_32_MODULUS)
    }
}
impl Sub for Field32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mod_value = FIELD_32_MODULUS;
        Self::new((self.value + mod_value - other.value) % mod_value)
    }
}
impl Mul for Field32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut res = self.value as usize * other.value as usize;
        res = res % FIELD_32_MODULUS as usize;
        Self::new(res as u32)
    }
}
impl Div for Field32 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            value: (self.value / other.value) % FIELD_32_MODULUS,
        }
    }
}
impl Neg for Field32 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(FIELD_32_MODULUS - self.value)
    }
}
impl Product<Field32> for Field32 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Field32>,
    {
        iter.into_iter()
            .fold(Field32 { value: 1 }, |acc, item| acc * item)
    }
}
impl Sum<Field32> for Field32 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Field32>,
    {
        iter.into_iter()
            .fold(Field32 { value: 0 }, |acc, item| acc + item)
    }
}

// Assign
impl AddAssign for Field32 {
    fn add_assign(&mut self, other: Field32) {
        // Add the values and reduce modulo `modulus`
        self.value = (self.value + other.value) % FIELD_32_MODULUS;
    }
}
impl SubAssign for Field32 {
    fn sub_assign(&mut self, other: Field32) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.value >= other.value {
            self.value = (self.value - other.value) % FIELD_32_MODULUS;
        } else {
            self.value = (self.value + FIELD_32_MODULUS - other.value) % FIELD_32_MODULUS;
        }
    }
}
impl MulAssign for Field32 {
    fn mul_assign(&mut self, other: Field32) {
        // Multiply the values and reduce modulo `modulus`
        self.value = (self.value * other.value) % FIELD_32_MODULUS;
    }
}
impl DivAssign for Field32 {
    fn div_assign(&mut self, other: Field32) {
        if other.value != 0 {
            self.value = (self.value / other.value) % FIELD_32_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

// left is a reference
impl<'a> Add<&'a Field32> for Field32 {
    type Output = Field32;

    fn add(self, rhs: &'a Field32) -> Self::Output {
        Self {
            value: (self.value + rhs.value) % FIELD_32_MODULUS,
        }
    }
}
impl<'a> Sub<&'a Field32> for Field32 {
    type Output = Field32;

    fn sub(self, rhs: &'a Field32) -> Self::Output {
        Self {
            value: (self.value - rhs.value) % FIELD_32_MODULUS,
        }
    }
}
impl<'a> Mul<&'a Field32> for Field32 {
    type Output = Field32;

    fn mul(self, rhs: &'a Field32) -> Self::Output {
        Self {
            value: (self.value * rhs.value) % FIELD_32_MODULUS,
        }
    }
}
impl<'a> Div<&'a Field32> for Field32 {
    type Output = Field32;

    fn div(self, rhs: &'a Field32) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self {
            value: (self.value / rhs.value) % FIELD_32_MODULUS,
        }
    }
}
impl<'a> Product<&'a Field32> for Field32 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Field32>,
    {
        iter.into_iter()
            .fold(Field32 { value: 1 }, |acc, item| acc * item)
    }
}
impl<'a> Sum<&'a Field32> for Field32 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Field32>,
    {
        iter.into_iter()
            .fold(Field32 { value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> DivAssign<&'a Field32> for Field32 {
    fn div_assign(&mut self, other: &'a Field32) {
        if other.value != 0 {
            self.value = (self.value / other.value) % FIELD_32_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

// I literally don't know, but it wants this too
impl Add<&mut Field32> for Field32 {
    type Output = Field32;

    fn add(self, other: &mut Field32) -> Field32 {
        Field32 {
            value: (self.value + other.value) % FIELD_32_MODULUS,
        }
    }
}
impl Sub<&mut Field32> for Field32 {
    type Output = Field32;

    fn sub(self, other: &mut Field32) -> Field32 {
        Field32 {
            value: (self.value - other.value) % FIELD_32_MODULUS,
        }
    }
}
impl Mul<&mut Field32> for Field32 {
    type Output = Field32;

    fn mul(self, rhs: &mut Field32) -> Self::Output {
        Self {
            value: (self.value * rhs.value) % self.value,
        }
    }
}
impl Div<&mut Field32> for Field32 {
    type Output = Field32;

    fn div(self, rhs: &mut Field32) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self {
            value: (self.value / rhs.value) % self.value,
        }
    }
}
