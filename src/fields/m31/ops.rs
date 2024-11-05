use ark_ff::Field;
use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::m31::{M31, M31_MODULUS};

// fn add_mod<M31>(a: M31, b: M31) -> M31
// {
//     let sum =
// }

// std::ops by value
impl Add for M31 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from((self.value + rhs.value) % M31_MODULUS)
    }
}
impl Sub for M31 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + M31_MODULUS - rhs.value) % M31_MODULUS);
        }
        Self::from((self.value - rhs.value) % M31_MODULUS)
    }
}
impl Mul for M31 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut product = self.to_u64() * rhs.to_u64();
        product = (product & M31_MODULUS as u64) + (product >> 31);
        product = (product & M31_MODULUS as u64) + (product >> 31);
        Self::from(product as u32)
    }
}
impl Div for M31 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % M31_MODULUS as u64)
                as u32,
        }
    }
}

// std::ops by lifetimed reference
impl<'a> Add<&'a M31> for M31 {
    type Output = Self;
    fn add(self, rhs: &'a Self) -> Self {
        Self::from((self.value + rhs.value) % M31_MODULUS)
    }
}
impl<'a> Sub<&'a M31> for M31 {
    type Output = Self;
    fn sub(self, rhs: &'a Self) -> Self {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + M31_MODULUS - rhs.value) % M31_MODULUS);
        }
        Self::from((self.value - rhs.value) % M31_MODULUS)
    }
}
impl<'a> Mul<&'a M31> for M31 {
    type Output = Self;
    fn mul(self, other: &'a Self) -> Self {
        Self::from(((self.value as u64 * other.value as u64) % M31_MODULUS as u64) as u32)
    }
}
impl<'a> Div<&'a M31> for M31 {
    type Output = Self;
    fn div(self, rhs: &'a Self) -> Self {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % M31_MODULUS as u64)
                as u32,
        }
    }
}

// std::ops by mut reference (NOTE: not the same as OpAssign below)
impl Add<&mut M31> for M31 {
    type Output = M31;
    fn add(self, rhs: &mut Self) -> Self::Output {
        Self::from((self.value + rhs.value) % M31_MODULUS)
    }
}
impl Sub<&mut M31> for M31 {
    type Output = M31;
    fn sub(self, rhs: &mut Self) -> Self::Output {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + M31_MODULUS - rhs.value) % M31_MODULUS);
        }
        Self::from((self.value - rhs.value) % M31_MODULUS)
    }
}
impl Mul<&mut M31> for M31 {
    type Output = M31;
    fn mul(self, rhs: &mut Self) -> Self::Output {
        Self::from(((self.value as u64 * rhs.value as u64) % M31_MODULUS as u64) as u32)
    }
}
impl Div<&mut M31> for M31 {
    type Output = M31;
    fn div(self, rhs: &mut Self) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % M31_MODULUS as u64)
                as u32,
        }
    }
}

// std::AssignOp by mut reference
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

impl<'a> AddAssign<&'a mut M31> for M31 {
    fn add_assign(&mut self, other: &'a mut M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS;
    }
}
impl<'a> SubAssign<&'a mut M31> for M31 {
    fn sub_assign(&mut self, rhs: &'a mut M31) {
        if self.value < rhs.value {
            // add the modulus
            self.value = (self.value + M31_MODULUS - rhs.value) % M31_MODULUS;
        } else {
            self.value = (self.value - rhs.value) % M31_MODULUS;
        }
    }
}
impl<'a> MulAssign<&'a mut M31> for M31 {
    fn mul_assign(&mut self, other: &'a mut M31) {
        self.value = ((self.value as u64 * other.value as u64) % M31_MODULUS as u64) as u32;
    }
}
impl<'a> DivAssign<&'a mut M31> for M31 {
    fn div_assign(&mut self, rhs: &'a mut M31) {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        self.value =
            ((self.value as u64 * rhs.inverse().unwrap().value as u64) % M31_MODULUS as u64) as u32;
    }
}

impl<'a> AddAssign<&'a M31> for M31 {
    fn add_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_add(other.value)) % M31_MODULUS;
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
impl<'a> MulAssign<&'a M31> for M31 {
    fn mul_assign(&mut self, other: &'a M31) {
        self.value = (self.value.wrapping_mul(other.value)) % M31_MODULUS;
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

#[cfg(test)]
mod tests {
    use crate::fields::m31::{M31, M31_MODULUS};
    use ark_ff::Field;

    #[test]
    fn test_add() {
        // basic
        let a = M31::from(10);
        let b = M31::from(22);
        assert_eq!(a + b, M31::from(32));
        // larger than modulus
        let c = M31::from(M31_MODULUS);
        let d = M31::from(1);
        assert_eq!(c + d, M31::from(1));
        // doesn't overflow
        let e = M31::from(u32::MAX - 2);
        let f = M31::from(3);
        assert_eq!(e + f, M31::from(2));
        // doesn't overflow
        let g = M31::from(M31_MODULUS - 1);
        let h = M31::from(M31_MODULUS - 1);
        assert_eq!(g + h, M31::from(2147483645));
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
        let c = M31::from(M31_MODULUS);
        let d = M31::from(M31_MODULUS);
        assert_eq!(c * d, M31::from(4611686014132420609_u64));
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
        assert_eq!(d.inverse().unwrap(), M31::from(1431655765));
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
