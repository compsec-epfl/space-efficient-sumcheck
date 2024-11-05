use ark_ff::{Field, Zero};
use ark_serialize::Flags;

use crate::fields::m31::{M31, M31_MODULUS};

impl Field for M31 {
    type BasePrimeField = Self;

    type BasePrimeFieldIter = std::iter::Empty<Self>;

    const SQRT_PRECOMP: Option<ark_ff::SqrtPrecomputation<Self>> = None; // what values should I precompute?

    const ZERO: Self = Self { value: 0 };

    const ONE: Self = Self { value: 1 };

    fn double(&self) -> Self {
        M31::from((2 * self.value) % M31_MODULUS)
    }

    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        let x = *self;
        let y = x.exp_power_of_2(2) * x;
        let z = y.square() * y;
        let a = z.exp_power_of_2(4) * z;
        let b = a.exp_power_of_2(4);
        let c = b * z;
        let d = b.exp_power_of_2(4) * a;
        let e = d.exp_power_of_2(12) * c;
        let f = e.exp_power_of_2(3) * y;
        Some(f)
    }

    fn frobenius_map(&self, _: usize) -> M31 {
        Self { value: self.value }
    }

    fn extension_degree() -> u64 {
        todo!()
    }

    fn to_base_prime_field_elements(&self) -> Self::BasePrimeFieldIter {
        todo!()
    }

    fn from_base_prime_field_elems(_elems: &[Self::BasePrimeField]) -> Option<Self> {
        todo!()
    }

    fn from_base_prime_field(_elem: Self::BasePrimeField) -> Self {
        todo!()
    }

    fn double_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn neg_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn from_random_bytes_with_flags<F: Flags>(_bytes: &[u8]) -> Option<(Self, F)> {
        todo!()
    }

    fn legendre(&self) -> ark_ff::LegendreSymbol {
        todo!()
    }

    fn square(&self) -> Self {
        self.clone() * self.clone()
    }

    fn square_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        todo!()
    }

    fn frobenius_map_in_place(&mut self, _power: usize) {
        todo!()
    }

    fn characteristic() -> &'static [u64] {
        &[]
    }

    fn from_random_bytes(_bytes: &[u8]) -> Option<Self> {
        std::unimplemented!()
    }

    fn sqrt(&self) -> Option<Self> {
        std::unimplemented!()
    }

    fn sqrt_in_place(&mut self) -> Option<&mut Self> {
        std::unimplemented!()
    }

    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }

    fn pow<S: AsRef<[u64]>>(&self, _exp: S) -> Self {
        *self
    }

    fn pow_with_table<S: AsRef<[u64]>>(_powers_of_2: &[Self], _exp: S) -> Option<Self> {
        std::unimplemented!()
    }
}
