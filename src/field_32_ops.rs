use ark_std::{iter::{Product, Sum}, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}};

use ark_ff::Field;

use crate::field_32::{Field32, FIELD_32_MODULUS};

// impl<'a> Mul<&'a Field32> for Field32 {
//     type Output = Field32;

//     fn mul(self, other: &Field32) -> Field32 {
//         Field32 {
//             value: self.value * other.value,
//             modulus: FIELD_32_MODULUS,
//         }
//     }
// }

// Basic
impl Add for Field32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new((self.value + other.value) % self.modulus, self.modulus)
    }
}
impl Sub for Field32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mod_value = self.modulus;
        Self::new((self.value + mod_value - other.value) % mod_value, mod_value)
    }
}
impl Mul for Field32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new((self.value * other.value) % self.modulus, self.modulus)
    }
}
impl Div for Field32 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self { value: (self.value / other.value) % self.modulus, modulus: self.modulus }
    }
}
impl Neg for Field32 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(self.modulus - self.value, self.modulus)
    }
}
impl Product<Field32> for Field32 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Field32>,
    {
        iter.into_iter().fold(
            Field32 { value: 1, modulus: FIELD_32_MODULUS },
            |acc, item| acc * item,
        )
    }
}
impl Sum<Field32> for Field32 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Field32>,
    {
        iter.into_iter().fold(
            Field32 { value: 0, modulus: FIELD_32_MODULUS },
            |acc, item| acc + item,
        )
    }
}

// Assign
impl AddAssign for Field32 {
    fn add_assign(&mut self, other: Field32) {
        // Add the values and reduce modulo `modulus`
        self.value = (self.value + other.value) % self.modulus;
    }
}
impl SubAssign for Field32 {
    fn sub_assign(&mut self, other: Field32) {
        // Perform subtraction and ensure it's non-negative by adding modulus if necessary
        if self.value >= other.value {
            self.value = (self.value - other.value) % self.modulus;
        } else {
            self.value = (self.value + self.modulus - other.value) % self.modulus;
        }
    }
}
impl MulAssign for Field32 {
    fn mul_assign(&mut self, other: Field32) {
        // Multiply the values and reduce modulo `modulus`
        self.value = (self.value * other.value) % self.modulus;
    }
}
impl DivAssign for Field32 {
    fn div_assign(&mut self, other: Field32) {
        if other.value != 0 {
            self.value = (self.value / other.value) % self.modulus;
        } else {
            panic!("Division by zero or no modular inverse exists");
        }
    }
}

// left is a reference
impl<'a> Add<&'a Field32> for Field32 {
    type Output = Field32;

    fn add(self, rhs: &'a Field32) -> Self::Output {
        Self { value: (self.value + rhs.value) % self.modulus, modulus: self.modulus }
    }
}
impl<'a> Sub<&'a Field32> for Field32 {
    type Output = Field32;

    fn sub(self, rhs: &'a Field32) -> Self::Output {
        Self { value: (self.value - rhs.value) % self.modulus, modulus: self.modulus }
    }
}
impl<'a> Mul<&'a Field32> for Field32 {
    type Output = Field32;

    fn mul(self, rhs: &'a Field32) -> Self::Output {
        Self { value: (self.value * rhs.value) % self.value, modulus: self.modulus }
    }
}
impl<'a> Div<&'a Field32> for Field32 {
    type Output = Field32;

    fn div(self, rhs: &'a Field32) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self { value: (self.value / rhs.value) % self.value, modulus: self.value }
    }
}
impl<'a> Product<&'a Field32> for Field32 {
    fn product<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Field32>,
    {
        iter.into_iter().fold(
            Field32 { value: 1, modulus: FIELD_32_MODULUS },
            |acc, item| acc * item,
        )
    }
}
impl<'a> Sum<&'a Field32> for Field32 {
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Field32>,
    {
        iter.into_iter().fold(
            Field32 { value: 0, modulus: FIELD_32_MODULUS },
            |acc, item| acc + item,
        )
    }
}

impl<'a> DivAssign<&'a Field32> for Field32 {
    fn div_assign(&mut self, other: &'a Field32) {
        if other.value != 0 {
            self.value = (self.value / other.value) % self.modulus;
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
            value: (self.value + other.value) % self.modulus, modulus: self.modulus,
        }
    }
}
impl Sub<&mut Field32> for Field32 {
    type Output = Field32;

    fn sub(self, other: &mut Field32) -> Field32 {
        Field32 {
            value: (self.value - other.value) % self.modulus, modulus: self.modulus,
        }
    }
}
impl Mul<&mut Field32> for Field32 {
    type Output = Field32;

    fn mul(self, rhs: &mut Field32) -> Self::Output {
        Self { value: (self.value * rhs.value) % self.value, modulus: self.modulus }
    }
}
impl Div<&mut Field32> for Field32 {
    type Output = Field32;

    fn div(self, rhs: &mut Field32) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero or no modular inverse exists");
        }
        Self { value: (self.value / rhs.value) % self.value, modulus: self.value }
    }
}