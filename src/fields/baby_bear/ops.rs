use ark_ff::Field;
use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::baby_bear::{BabyBear, BB_MODULUS_U32, BB_MODULUS_U64};

// by value
impl Add for BabyBear {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from((self.value + rhs.value) % BB_MODULUS_U32)
    }
}
impl Sub for BabyBear {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + BB_MODULUS_U32 - rhs.value) % BB_MODULUS_U32);
        }
        Self::from((self.value - rhs.value) % BB_MODULUS_U32)
    }
}
impl Mul for BabyBear {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::from(((self.value as u64 * other.value as u64) % BB_MODULUS_U64) as u32)
    }
}
impl Div for BabyBear {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % BB_MODULUS_U64)
                as u32,
        }
    }
}

// by reference
impl<'a> Add<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn add(self, rhs: &'a Self) -> Self {
        Self::from((self.value + rhs.value) % BB_MODULUS_U32)
    }
}
impl<'a> Sub<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn sub(self, rhs: &'a Self) -> Self {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + BB_MODULUS_U32 - rhs.value) % BB_MODULUS_U32);
        }
        Self::from((self.value - rhs.value) % BB_MODULUS_U32)
    }
}
impl<'a> Mul<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn mul(self, other: &'a Self) -> Self {
        Self::from(((self.value as u64 * other.value as u64) % BB_MODULUS_U64) as u32)
    }
}
impl<'a> Div<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn div(self, rhs: &'a Self) -> Self {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % BB_MODULUS_U64)
                as u32,
        }
    }
}

// by mut reference
impl Add<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn add(self, rhs: &mut Self) -> Self::Output {
        Self::from((self.value + rhs.value) % BB_MODULUS_U32)
    }
}
impl Sub<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn sub(self, rhs: &mut Self) -> Self::Output {
        if self.value < rhs.value {
            // add the modulus
            return Self::from((self.value + BB_MODULUS_U32 - rhs.value) % BB_MODULUS_U32);
        }
        Self::from((self.value - rhs.value) % BB_MODULUS_U32)
    }
}
impl Mul<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn mul(self, rhs: &mut Self) -> Self::Output {
        Self::from(((self.value as u64 * rhs.value as u64) % BB_MODULUS_U64) as u32)
    }
}
impl Div<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn div(self, rhs: &mut Self) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        Self {
            value: ((self.value as u64 * rhs.inverse().unwrap().value as u64) % BB_MODULUS_U64)
                as u32,
        }
    }
}

// TODO below unchecked

// Assign by mut reference
impl AddAssign for BabyBear {
    fn add_assign(&mut self, other: BabyBear) {
        // Add the values and reduce modulo `modulus`
        self.value = (self.value + other.value) % BB_MODULUS_U32;
    }
}
impl SubAssign for BabyBear {
    fn sub_assign(&mut self, other: BabyBear) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.value >= other.value {
            self.value = (self.value - other.value) % BB_MODULUS_U32;
        } else {
            self.value = (self.value + BB_MODULUS_U32 - other.value) % BB_MODULUS_U32;
        }
    }
}
impl MulAssign for BabyBear {
    fn mul_assign(&mut self, other: BabyBear) {
        // Multiply the values and reduce modulo `modulus`
        self.value = (self.value * other.value) % BB_MODULUS_U32;
    }
}
impl DivAssign for BabyBear {
    fn div_assign(&mut self, other: BabyBear) {
        if other.value != 0 {
            self.value = (self.value / other.value) % BB_MODULUS_U32;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

impl<'a> AddAssign<&'a mut BabyBear> for BabyBear {
    fn add_assign(&mut self, other: &'a mut BabyBear) {
        self.value = (self.value.wrapping_add(other.value)) % BB_MODULUS_U32;
    }
}
impl<'a> SubAssign<&'a mut BabyBear> for BabyBear {
    fn sub_assign(&mut self, rhs: &'a mut BabyBear) {
        if self.value < rhs.value {
            // add the modulus
            self.value = (self.value + BB_MODULUS_U32 - rhs.value) % BB_MODULUS_U32;
        } else {
            self.value = (self.value - rhs.value) % BB_MODULUS_U32;
        }
    }
}
impl<'a> MulAssign<&'a mut BabyBear> for BabyBear {
    fn mul_assign(&mut self, other: &'a mut BabyBear) {
        self.value = ((self.value as u64 * other.value as u64) % BB_MODULUS_U64) as u32;
    }
}
impl<'a> DivAssign<&'a mut BabyBear> for BabyBear {
    fn div_assign(&mut self, rhs: &'a mut BabyBear) {
        if rhs.value == 0 {
            panic!("Division by zero");
        }
        self.value =
            ((self.value as u64 * rhs.inverse().unwrap().value as u64) % BB_MODULUS_U64) as u32;
    }
}

impl<'a> AddAssign<&'a BabyBear> for BabyBear {
    fn add_assign(&mut self, other: &'a BabyBear) {
        self.value = (self.value.wrapping_add(other.value)) % BB_MODULUS_U32;
    }
}
impl<'a> SubAssign<&'a BabyBear> for BabyBear {
    fn sub_assign(&mut self, other: &'a BabyBear) {
        self.value = (self.value.wrapping_sub(other.value)) % BB_MODULUS_U32;

        // Handle negative results by adding modulus
        if self.value > BB_MODULUS_U32 {
            self.value += BB_MODULUS_U32;
        }
    }
}
impl<'a> MulAssign<&'a BabyBear> for BabyBear {
    fn mul_assign(&mut self, other: &'a BabyBear) {
        self.value = (self.value.wrapping_mul(other.value)) % BB_MODULUS_U32;
    }
}

impl Neg for BabyBear {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from(BB_MODULUS_U32 - self.value)
    }
}
impl Product<BabyBear> for BabyBear {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { value: 1 }, |acc, item| acc * item)
    }
}
impl Sum<BabyBear> for BabyBear {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> Product<&'a BabyBear> for BabyBear {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { value: 1 }, |acc, item| acc * item)
    }
}
impl<'a> Sum<&'a BabyBear> for BabyBear {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> DivAssign<&'a BabyBear> for BabyBear {
    fn div_assign(&mut self, other: &'a BabyBear) {
        if other.value != 0 {
            self.value = (self.value / other.value) % BB_MODULUS_U32;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::baby_bear::{BabyBear, BB_MODULUS_U32};
    use ark_ff::Field;

    #[test]
    fn test_add() {
        // basic
        let a = BabyBear::from(10);
        let b = BabyBear::from(22);
        assert_eq!(a + b, BabyBear::from(32));
        // larger than modulus
        let c = BabyBear::from(BB_MODULUS_U32);
        let d = BabyBear::from(1);
        assert_eq!(c + d, BabyBear::from(1));
        // doesn't overflow
        let e = BabyBear::from(u32::MAX - 2);
        let f = BabyBear::from(3);
        assert_eq!(e + f, BabyBear::from(268435454));
        // doesn't overflow
        let g = BabyBear::from(BB_MODULUS_U32 - 1);
        let h = BabyBear::from(BB_MODULUS_U32 - 1);
        assert_eq!(g + h, BabyBear::from(2013265919));
    }

    #[test]
    fn test_sub() {
        // basic
        let a = BabyBear::from(22);
        let b = BabyBear::from(10);
        assert_eq!(a - b, BabyBear::from(12));
        // doesn't underflow
        let c = BabyBear::from(10);
        let d = BabyBear::from(22);
        assert_eq!(c - d, BabyBear::from(2013265909));
    }

    #[test]
    fn test_mul() {
        // basic
        let a = BabyBear::from(10);
        let b = BabyBear::from(22);
        assert_eq!(a * b, BabyBear::from(220));
        // doesn't overflow
        let c = BabyBear::from(BB_MODULUS_U32 - 1);
        let d = BabyBear::from(30000);
        assert_eq!(c * d, BabyBear::from(2013235921));
    }

    #[test]
    fn test_div() {
        // basic
        let a = BabyBear::from(10);
        let b = BabyBear::from(2);
        assert_eq!(a / b, BabyBear::from(5));
        // not divisor
        let c = BabyBear::from(10);
        let d = BabyBear::from(3);
        assert_eq!(d.inverse().unwrap(), BabyBear::from(1342177281));
        assert_eq!((c / d) * d, BabyBear::from(10));
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero() {
        let a = BabyBear::from(10);
        let b = BabyBear::from(0);
        let _result = a / b;
    }

    // #[test]
    // fn test_neg() {
    //     let a = BabyBear::from(10);
    //     let result = -a;
    //     assert_eq!(result, BabyBear::from(21));
    // }

    // #[test]
    // fn test_add_assign() {
    //     let mut a = BabyBear::from(10);
    //     let b = BabyBear::from(22);
    //     a += b;
    //     assert_eq!(a, BabyBear::from(1));
    // }

    // #[test]
    // fn test_sub_assign() {
    //     let mut a = BabyBear::from(10);
    //     let b = BabyBear::from(22);
    //     a -= b;
    //     assert_eq!(a, BabyBear::from(19));
    // }

    // #[test]
    // fn test_mul_assign() {
    //     let mut a = BabyBear::from(10);
    //     let b = BabyBear::from(22);
    //     a *= b;
    //     assert_eq!(a, BabyBear::from(3));
    // }

    // #[test]
    // fn test_div_assign() {
    //     let mut a = BabyBear::from(10);
    //     let b = BabyBear::from(2);
    //     a /= b;
    //     assert_eq!(a, BabyBear::from(5));
    // }

    // #[test]
    // #[should_panic(expected = "Division by zero or no modular inverse exists")]
    // fn test_div_assign_by_zero() {
    //     let mut a = BabyBear::from(10);
    //     let b = BabyBear::from(0);
    //     a /= b;
    // }

    // #[test]
    // fn test_product() {
    //     let values = vec![BabyBear::from(10), BabyBear::from(2), BabyBear::from(3)];
    //     let result: BabyBear = values.into_iter().product();
    //     assert_eq!(result, BabyBear::from(20));
    // }

    // #[test]
    // fn test_sum() {
    //     let values = vec![BabyBear::from(10), BabyBear::from(2), BabyBear::from(3)];
    //     let result: BabyBear = values.into_iter().sum();
    //     assert_eq!(result, BabyBear::from(15));
    // }
}
