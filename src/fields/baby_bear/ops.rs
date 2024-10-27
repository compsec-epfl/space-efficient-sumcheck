use ark_ff::Field;
use ark_std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::fields::baby_bear::{
    transmute::mod_transmute_unsigned, BabyBear, BB_MODULUS, BB_MODULUS_U64,
};

impl Add for BabyBear {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            mod_value: mod_transmute_unsigned(self.to_u32() + rhs.to_u32(), BB_MODULUS),
        }
    }
}
impl Sub for BabyBear {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            mod_value: match self.mod_value < rhs.mod_value {
                true => {
                    mod_transmute_unsigned(BB_MODULUS + self.to_u32() - rhs.to_u32(), BB_MODULUS)
                }
                false => mod_transmute_unsigned(self.to_u32() - rhs.to_u32(), BB_MODULUS),
            },
        }
    }
}
impl Mul for BabyBear {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            mod_value: mod_transmute_unsigned(self.to_u64() * rhs.to_u64(), BB_MODULUS_U64),
        }
    }
}
impl Div for BabyBear {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.mod_value == 0 {
            panic!("Division by zero");
        }
        Self {
            mod_value: mod_transmute_unsigned(
                self.to_u64() * rhs.inverse().unwrap().to_u64(),
                BB_MODULUS_U64,
            ),
        }
    }
}

// by reference
impl<'a> Add<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn add(self, rhs: &'a Self) -> Self {
        Self::from((self.mod_value + rhs.mod_value) % BB_MODULUS)
    }
}
impl<'a> Sub<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn sub(self, rhs: &'a Self) -> Self {
        if self.mod_value < rhs.mod_value {
            // add the modulus
            return Self::from((self.mod_value + BB_MODULUS - rhs.mod_value) % BB_MODULUS);
        }
        Self::from((self.mod_value - rhs.mod_value) % BB_MODULUS)
    }
}
impl<'a> Mul<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn mul(self, rhs: &'a Self) -> Self {
        Self::from(((self.mod_value as u64 * rhs.mod_value as u64) % BB_MODULUS_U64) as u32)
    }
}
impl<'a> Div<&'a BabyBear> for BabyBear {
    type Output = Self;
    fn div(self, rhs: &'a Self) -> Self {
        if rhs.mod_value == 0 {
            panic!("Division by zero");
        }
        Self {
            mod_value: ((self.mod_value as u64 * rhs.inverse().unwrap().mod_value as u64)
                % BB_MODULUS_U64) as u32,
        }
    }
}

// by mut reference
impl Add<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn add(self, rhs: &mut Self) -> Self::Output {
        Self::from((self.mod_value + rhs.mod_value) % BB_MODULUS)
    }
}
impl Sub<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn sub(self, rhs: &mut Self) -> Self::Output {
        if self.mod_value < rhs.mod_value {
            // add the modulus
            return Self::from((self.mod_value + BB_MODULUS - rhs.mod_value) % BB_MODULUS);
        }
        Self::from((self.mod_value - rhs.mod_value) % BB_MODULUS)
    }
}
impl Mul<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn mul(self, rhs: &mut Self) -> Self::Output {
        Self::from(((self.mod_value as u64 * rhs.mod_value as u64) % BB_MODULUS_U64) as u32)
    }
}
impl Div<&mut BabyBear> for BabyBear {
    type Output = BabyBear;
    fn div(self, rhs: &mut Self) -> Self::Output {
        if rhs.mod_value == 0 {
            panic!("Division by zero");
        }
        Self {
            mod_value: ((self.mod_value as u64 * rhs.inverse().unwrap().mod_value as u64)
                % BB_MODULUS_U64) as u32,
        }
    }
}

// TODO below unchecked

// Assign by mut reference
impl AddAssign for BabyBear {
    fn add_assign(&mut self, rhs: BabyBear) {
        // Add the values and reduce modulo `modulus`
        self.mod_value = (self.mod_value + rhs.mod_value) % BB_MODULUS;
    }
}
impl SubAssign for BabyBear {
    fn sub_assign(&mut self, rhs: BabyBear) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.mod_value >= rhs.mod_value {
            self.mod_value = (self.mod_value - rhs.mod_value) % BB_MODULUS;
        } else {
            self.mod_value = (self.mod_value + BB_MODULUS - rhs.mod_value) % BB_MODULUS;
        }
    }
}
impl MulAssign for BabyBear {
    fn mul_assign(&mut self, rhs: BabyBear) {
        // Multiply the values and reduce modulo `modulus`
        self.mod_value = (self.mod_value * rhs.mod_value) % BB_MODULUS;
    }
}
impl DivAssign for BabyBear {
    fn div_assign(&mut self, rhs: BabyBear) {
        if rhs.mod_value != 0 {
            self.mod_value = (self.mod_value / rhs.mod_value) % BB_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

impl<'a> AddAssign<&'a mut BabyBear> for BabyBear {
    fn add_assign(&mut self, rhs: &'a mut BabyBear) {
        self.mod_value = (self.mod_value.wrapping_add(rhs.mod_value)) % BB_MODULUS;
    }
}
impl<'a> SubAssign<&'a mut BabyBear> for BabyBear {
    fn sub_assign(&mut self, rhs: &'a mut BabyBear) {
        if self.mod_value < rhs.mod_value {
            // add the modulus
            self.mod_value = (self.mod_value + BB_MODULUS - rhs.mod_value) % BB_MODULUS;
        } else {
            self.mod_value = (self.mod_value - rhs.mod_value) % BB_MODULUS;
        }
    }
}
impl<'a> MulAssign<&'a mut BabyBear> for BabyBear {
    fn mul_assign(&mut self, rhs: &'a mut BabyBear) {
        self.mod_value = ((self.mod_value as u64 * rhs.mod_value as u64) % BB_MODULUS_U64) as u32;
    }
}
impl<'a> DivAssign<&'a mut BabyBear> for BabyBear {
    fn div_assign(&mut self, rhs: &'a mut BabyBear) {
        if rhs.mod_value == 0 {
            panic!("Division by zero");
        }
        self.mod_value = ((self.mod_value as u64 * rhs.inverse().unwrap().mod_value as u64)
            % BB_MODULUS_U64) as u32;
    }
}

impl<'a> AddAssign<&'a BabyBear> for BabyBear {
    fn add_assign(&mut self, rhs: &'a BabyBear) {
        self.mod_value = (self.mod_value.wrapping_add(rhs.mod_value)) % BB_MODULUS;
    }
}
impl<'a> SubAssign<&'a BabyBear> for BabyBear {
    fn sub_assign(&mut self, rhs: &'a BabyBear) {
        self.mod_value = (self.mod_value.wrapping_sub(rhs.mod_value)) % BB_MODULUS;

        // Handle negative results by adding modulus
        if self.mod_value > BB_MODULUS {
            self.mod_value += BB_MODULUS;
        }
    }
}
impl<'a> MulAssign<&'a BabyBear> for BabyBear {
    fn mul_assign(&mut self, rhs: &'a BabyBear) {
        self.mod_value = (self.mod_value.wrapping_mul(rhs.mod_value)) % BB_MODULUS;
    }
}

impl Neg for BabyBear {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from(BB_MODULUS - self.mod_value)
    }
}
impl Product<BabyBear> for BabyBear {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { mod_value: 1 }, |acc, item| acc * item)
    }
}
impl Sum<BabyBear> for BabyBear {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { mod_value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> Product<&'a BabyBear> for BabyBear {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { mod_value: 1 }, |acc, item| acc * item)
    }
}
impl<'a> Sum<&'a BabyBear> for BabyBear {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a BabyBear>,
    {
        iter.into_iter()
            .fold(BabyBear { mod_value: 0 }, |acc, item| acc + item)
    }
}

impl<'a> DivAssign<&'a BabyBear> for BabyBear {
    fn div_assign(&mut self, other: &'a BabyBear) {
        if other.mod_value != 0 {
            self.mod_value = (self.mod_value / other.mod_value) % BB_MODULUS;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::baby_bear::{BabyBear, BB_MODULUS};
    use ark_ff::Field;

    #[test]
    fn test_add() {
        // basic
        let a = BabyBear::from(10);
        let b = BabyBear::from(22);
        assert_eq!(a + b, BabyBear::from(32));
        // larger than modulus
        let c = BabyBear::from(BB_MODULUS);
        let d = BabyBear::from(1);
        assert_eq!(c + d, BabyBear::from(1));
        // doesn't overflow
        let e = BabyBear::from(u32::MAX - 2);
        let f = BabyBear::from(3);
        assert_eq!(e + f, BabyBear::from(268435454));
        // doesn't overflow
        let g = BabyBear::from(BB_MODULUS - 1);
        let h = BabyBear::from(BB_MODULUS - 1);
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
        let c = BabyBear::from(BB_MODULUS - 1);
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
