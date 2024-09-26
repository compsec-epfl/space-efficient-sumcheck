use ark_ff::{Field, Zero};
use ark_serialize::Flags;

use crate::fields::baby_bear::BabyBear;

impl Field for BabyBear {
    type BasePrimeField = Self;

    type BasePrimeFieldIter = std::iter::Empty<Self>;

    const SQRT_PRECOMP: Option<ark_ff::SqrtPrecomputation<Self>> = None;

    const ZERO: Self = Self { mod_value: 0_u32 };

    const ONE: Self = Self { mod_value: 1_u32 };

    fn double(&self) -> Self {
        BabyBear::from(2_u32) * self
    }

    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        let t0 = self.exp_power_of_2(8);
        let t1 = t0 * self;
        let t2 = t0.exp_power_of_2(8);
        let t3 = t2 * t1;
        let t4 = t3.exp_power_of_2(3);
        let t5 = t4.exp_power_of_2(5);
        let t6 = t5 * self;
        let t7 = t6 * t4;
        let t8 = t6.square();
        let t9 = t8 * t7;
        let t10 = t8.square();
        let t11 = t10 * t9;
        let t12 = t11.exp_power_of_2(4);

        Some(t12 * t11)
    }

    fn frobenius_map(&self, _: usize) -> BabyBear {
        Self {
            mod_value: self.mod_value,
        }
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
        *self * self
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
