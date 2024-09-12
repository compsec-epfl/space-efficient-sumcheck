use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::m31::{M31, M31_MODULUS};

impl<'a> DivAssign<&'a mut M31> for M31 {
    fn div_assign(&mut self, other: &'a mut M31) {
        if other.value == 0 {
            panic!("Division by zero");
        }
        self.value = (self.value / other.value) % M31_MODULUS;
    }
}
impl<'a> MulAssign<&'a mut M31> for M31 {
    fn mul_assign(&mut self, other: &'a mut M31) {
        let mut tmp: u64 = self.value as u64 * other.value as u64;
        tmp = tmp % M31_MODULUS as u64;
        self.value = tmp as u32;
    }
}

impl<'a> SubAssign<&'a mut M31> for M31 {
    fn sub_assign(&mut self, other: &'a mut M31) {
        let mut tmp: f64 = self.value as f64 - other.value as f64;
        if tmp < 0_f64 {
            tmp = M31_MODULUS as f64 - tmp;
        } else {
            tmp = tmp % M31_MODULUS as f64;
        }
        self.value = tmp as u32;
    }
}
impl<'a> AddAssign<&'a mut M31> for M31 {
    fn add_assign(&mut self, other: &'a mut M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS;
    }
}
impl<'a> MulAssign<&'a M31> for M31 {
    fn mul_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_mul(other.value)) % M31_MODULUS;
    }
}
impl<'a> SubAssign<&'a M31> for M31 {
    fn sub_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_sub(other.value)) % M31_MODULUS;

        // Handle negative results by adding modulus
        if self.value > M31_MODULUS {
            self.value += M31_MODULUS;
        }
    }
}

impl<'a> AddAssign<&'a M31> for M31 {
    fn add_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS;
    }
}

// Basic
impl Add for M31 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from((self.value + other.value) % M31_MODULUS)
    }
}
impl Sub for M31 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mod_value = M31_MODULUS;
        Self::from((self.value + mod_value - other.value) % mod_value)
    }
}
impl Mul for M31 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut res = self.value as usize * other.value as usize;
        res = res % M31_MODULUS as usize;
        Self::from(res as u32)
    }
}
impl Div for M31 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            value: (self.value / other.value) % M31_MODULUS,
        }
    }
}
impl Neg for M31 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from(M31_MODULUS - self.value)
    }
}
impl Product<M31> for M31 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = M31>,
    {
        iter.into_iter()
            .fold(M31 { value: 1 }, |acc, item| acc * item)
    }
}
impl Sum<M31> for M31 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = M31>,
    {
        iter.into_iter()
            .fold(M31 { value: 0 }, |acc, item| acc + item)
    }
}

// Assign
impl AddAssign for M31 {
    fn add_assign(&mut self, other: M31) {
        // Add the values and reduce modulo `modulus`
        self.value = (self.value + other.value) % M31_MODULUS;
    }
}
impl SubAssign for M31 {
    fn sub_assign(&mut self, other: M31) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.value >= other.value {
            self.value = (self.value - other.value) % M31_MODULUS;
        } else {
            self.value = (self.value + M31_MODULUS - other.value) % M31_MODULUS;
        }
    }
}
impl MulAssign for M31 {
    fn mul_assign(&mut self, other: M31) {
        // Multiply the values and reduce modulo `modulus`
        self.value = (self.value * other.value) % M31_MODULUS;
    }
}
impl DivAssign for M31 {
    fn div_assign(&mut self, other: M31) {
        if other.value != 0 {
            self.value = (self.value / other.value) % M31_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

// left is a reference
impl<'a> Add<&'a M31> for M31 {
    type Output = M31;

    fn add(self, rhs: &'a M31) -> Self::Output {
        Self {
            value: (self.value + rhs.value) % M31_MODULUS,
        }
    }
}
impl<'a> Sub<&'a M31> for M31 {
    type Output = M31;

    fn sub(self, rhs: &'a M31) -> Self::Output {
        Self {
            value: (self.value - rhs.value) % M31_MODULUS,
        }
    }
}
impl<'a> Mul<&'a M31> for M31 {
    type Output = M31;

    fn mul(self, rhs: &'a M31) -> Self::Output {
        Self {
            value: (self.value * rhs.value) % M31_MODULUS,
        }
    }
}
impl<'a> Div<&'a M31> for M31 {
    type Output = M31;

    fn div(self, rhs: &'a M31) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self {
            value: (self.value / rhs.value) % M31_MODULUS,
        }
    }
}
impl<'a> Product<&'a M31> for M31 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a M31>,
    {
        iter.into_iter()
            .fold(M31 { value: 1 }, |acc, item| acc * item)
    }
}
impl<'a> Sum<&'a M31> for M31 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a M31>,
    {
        iter.into_iter()
            .fold(M31 { value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> DivAssign<&'a M31> for M31 {
    fn div_assign(&mut self, other: &'a M31) {
        if other.value != 0 {
            self.value = (self.value / other.value) % M31_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

// I literally don't know, but it wants this too
impl Add<&mut M31> for M31 {
    type Output = M31;

    fn add(self, other: &mut M31) -> M31 {
        M31 {
            value: (self.value + other.value) % M31_MODULUS,
        }
    }
}
impl Sub<&mut M31> for M31 {
    type Output = M31;

    fn sub(self, other: &mut M31) -> M31 {
        M31 {
            value: (self.value - other.value) % M31_MODULUS,
        }
    }
}
impl Mul<&mut M31> for M31 {
    type Output = M31;

    fn mul(self, rhs: &mut M31) -> Self::Output {
        Self {
            value: (self.value * rhs.value) % self.value,
        }
    }
}
impl Div<&mut M31> for M31 {
    type Output = M31;

    fn div(self, rhs: &mut M31) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self {
            value: (self.value / rhs.value) % self.value,
        }
    }
}
