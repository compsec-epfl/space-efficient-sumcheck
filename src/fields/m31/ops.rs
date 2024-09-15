use ark_ff::Field;
use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::m31::{M31, M31_MODULUS_I64, M31_MODULUS_U32, M31_MODULUS_U64};

impl<'a> DivAssign<&'a mut M31> for M31 {
    fn div_assign(&mut self, rhs: &'a mut M31) {
        if rhs.value == 0 {
            panic!("Attempted division by 0");
        }
        self.value = (self.value / rhs.value) % M31_MODULUS_U32;
    }
}
impl<'a> MulAssign<&'a mut M31> for M31 {
    fn mul_assign(&mut self, other: &'a mut M31) {
        let mut tmp: u64 = self.value as u64 * other.value as u64;
        tmp = tmp % M31_MODULUS_U64;
        self.value = tmp as u32;
    }
}

impl<'a> SubAssign<&'a mut M31> for M31 {
    fn sub_assign(&mut self, other: &'a mut M31) {
        let mut tmp: i64 = self.value as i64 - other.value as i64;
        if tmp < 0_i64 {
            tmp = M31_MODULUS_I64 - tmp;
        } else {
            tmp = tmp % M31_MODULUS_I64;
        }
        self.value = tmp as u32;
    }
}
impl<'a> AddAssign<&'a mut M31> for M31 {
    fn add_assign(&mut self, other: &'a mut M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS_U32;
    }
}
impl<'a> MulAssign<&'a M31> for M31 {
    fn mul_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_mul(other.value)) % M31_MODULUS_U32;
    }
}
impl<'a> SubAssign<&'a M31> for M31 {
    fn sub_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_sub(other.value)) % M31_MODULUS_U32;

        // Handle negative results by adding modulus
        if self.value > M31_MODULUS_U32 {
            self.value += M31_MODULUS_U32;
        }
    }
}

impl<'a> AddAssign<&'a M31> for M31 {
    fn add_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS_U32;
    }
}

// Basic
impl Add for M31 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from((self.value + other.value) % M31_MODULUS_U32)
    }
}
impl Sub for M31 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mod_value = M31_MODULUS_U32;
        Self::from((self.value + mod_value - other.value) % mod_value)
    }
}
impl Mul for M31 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut res = self.value as usize * other.value as usize;
        res = res % M31_MODULUS_U32 as usize;
        Self::from(res as u32)
    }
}
impl Div for M31 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * other.inverse().unwrap().value as u64) % M31_MODULUS_U64)
                as u32,
        }
    }
}
impl Neg for M31 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from(M31_MODULUS_U32 - self.value)
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
        self.value = (self.value + other.value) % M31_MODULUS_U32;
    }
}
impl SubAssign for M31 {
    fn sub_assign(&mut self, other: M31) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.value >= other.value {
            self.value = (self.value - other.value) % M31_MODULUS_U32;
        } else {
            self.value = (self.value + M31_MODULUS_U32 - other.value) % M31_MODULUS_U32;
        }
    }
}
impl MulAssign for M31 {
    fn mul_assign(&mut self, other: M31) {
        // Multiply the values and reduce modulo `modulus`
        self.value = (self.value * other.value) % M31_MODULUS_U32;
    }
}
impl DivAssign for M31 {
    fn div_assign(&mut self, other: M31) {
        if other.value != 0 {
            self.value = (self.value / other.value) % M31_MODULUS_U32;
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
            value: (self.value + rhs.value) % M31_MODULUS_U32,
        }
    }
}
impl<'a> Sub<&'a M31> for M31 {
    type Output = M31;

    fn sub(self, rhs: &'a M31) -> Self::Output {
        if self.value >= rhs.value {
            return Self {
                value: (self.value - rhs.value) % M31_MODULUS_U32,
            };
        }
        Self {
            value: ((self.value as u64 + M31_MODULUS_U64 - rhs.value as u64) % M31_MODULUS_U64)
                as u32,
        }
    }
}
impl<'a> Mul<&'a M31> for M31 {
    type Output = M31;

    fn mul(self, rhs: &'a M31) -> Self::Output {
        Self {
            value: (self.value * rhs.value) % M31_MODULUS_U32,
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
            value: (self.value / rhs.value) % M31_MODULUS_U32,
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
            self.value = (self.value / other.value) % M31_MODULUS_U32;
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
            value: (self.value + other.value) % M31_MODULUS_U32,
        }
    }
}
impl Sub<&mut M31> for M31 {
    type Output = M31;

    fn sub(self, other: &mut M31) -> M31 {
        M31 {
            value: (self.value - other.value) % M31_MODULUS_U32,
        }
    }
}
impl Mul<&mut M31> for M31 {
    type Output = M31;

    fn mul(self, rhs: &mut M31) -> Self::Output {
        Self {
            value: ((self.value as u64 * rhs.value as u64) % self.value as u64) as u32,
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

#[cfg(test)]
mod tests {
    use ark_ff::Field;

    use crate::fields::m31::{M31, M31_MODULUS_U32};

    #[test]
    fn test_add() {
        // basic
        let a = M31::from(10);
        let b = M31::from(22);
        assert_eq!(a + b, M31::from(32));
        // larger than modulus
        let c = M31::from(M31_MODULUS_U32);
        let d = M31::from(1);
        assert_eq!(c + d, M31::from(1));
        // doesn't overflow
        let e = M31::from(u32::MAX - 2); // this is mod n
        let f = M31::from(3);
        assert_eq!(e + f, M31::from(2));
    }

    #[test]
    fn test_sub() {
        // basic
        let a = M31::from(22);
        let b = M31::from(10);
        assert_eq!(a - b, M31::from(12));
        // doesn't underflow
        let c = M31::from(10);
        let d = M31::from(22);
        assert_eq!(c - d, M31::from(2147483635_u32));
    }

    #[test]
    fn test_mul() {
        // basic
        let a = M31::from(10);
        let b = M31::from(22);
        assert_eq!(a * b, M31::from(220));
        // doesn't overflow
        let c = M31::from(M31_MODULUS_U32);
        let d = M31::from(M31_MODULUS_U32);
        assert_eq!(c * d, M31::from(4611686014132420609_u64)); // incidentally this is 0
    }

    #[test]
    fn test_div() {
        // basic
        let a = M31::from(10);
        let b = M31::from(2);
        assert_eq!(a / b, M31::from(5));
        // not divisor
        let c = M31::from(10);
        let d = M31::from(3);
        assert_eq!(d.inverse().unwrap(), M31::from(1431655765)); // depends on modular inverse
        assert_eq!(c / d, M31::from(1431655768));
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero() {
        let a = M31::from(10);
        let b = M31::from(0);
        let _result = a / b;
    }

    // #[test]
    // fn test_neg() {
    //     let a = M31::from(10);
    //     let result = -a;
    //     assert_eq!(result, M31::from(21));
    // }

    // #[test]
    // fn test_add_assign() {
    //     let mut a = M31::from(10);
    //     let b = M31::from(22);
    //     a += b;
    //     assert_eq!(a, M31::from(1));
    // }

    // #[test]
    // fn test_sub_assign() {
    //     let mut a = M31::from(10);
    //     let b = M31::from(22);
    //     a -= b;
    //     assert_eq!(a, M31::from(19));
    // }

    // #[test]
    // fn test_mul_assign() {
    //     let mut a = M31::from(10);
    //     let b = M31::from(22);
    //     a *= b;
    //     assert_eq!(a, M31::from(3));
    // }

    // #[test]
    // fn test_div_assign() {
    //     let mut a = M31::from(10);
    //     let b = M31::from(2);
    //     a /= b;
    //     assert_eq!(a, M31::from(5));
    // }

    // #[test]
    // #[should_panic(expected = "Division by zero or no modular inverse exists")]
    // fn test_div_assign_by_zero() {
    //     let mut a = M31::from(10);
    //     let b = M31::from(0);
    //     a /= b;
    // }

    // #[test]
    // fn test_product() {
    //     let values = vec![M31::from(10), M31::from(2), M31::from(3)];
    //     let result: M31 = values.into_iter().product();
    //     assert_eq!(result, M31::from(20));
    // }

    // #[test]
    // fn test_sum() {
    //     let values = vec![M31::from(10), M31::from(2), M31::from(3)];
    //     let result: M31 = values.into_iter().sum();
    //     assert_eq!(result, M31::from(15));
    // }
}
