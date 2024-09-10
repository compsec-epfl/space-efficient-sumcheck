use ark_ff::{Field, One, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Flags};

pub const FIELD_32_MODULUS: u32 = 4294967291;

#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, CanonicalDeserialize, CanonicalSerialize)]
pub struct Field32 {
    pub value: u32,
    pub modulus: u32,
}

impl Field32 {
    pub fn new(value: u32, modulus: u32) -> Self {
        Field32 {
            value: value % modulus,
            modulus,
        }
    }
}

impl Zero for Field32 {
    fn zero() -> Self {
        Field32::new(0, FIELD_32_MODULUS)
    }

    fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl One for Field32 {
    fn one() -> Self {
        Field32::new(1, FIELD_32_MODULUS)
    }
}

// Implement the Field trait
impl Field for Field32 {

    type BasePrimeField = Self;
    
    type BasePrimeFieldIter = std::iter::Empty<Self>;

    const SQRT_PRECOMP: Option<ark_ff::SqrtPrecomputation<Self>> = None;
    
    const ZERO: Self = Self { value: 0, modulus: FIELD_32_MODULUS };
    
    const ONE: Self = Self { value: 1, modulus: FIELD_32_MODULUS };

    fn double(&self) -> Self {
        Field32::new((2 * self.value) % self.modulus, self.modulus)
    }

    fn inverse(&self) -> Option<Self> {
        if self.value == 0 {
            return None
        }
        Some(Self::new((1 / self.value) % self.modulus, self.modulus))
    }

    fn frobenius_map(&self, _: usize) -> Field32 {
        // This is a no-op for prime fields
        Self { value: self.value, modulus: self.modulus }
    }
    
    fn extension_degree() -> u64 {
        todo!()
    }
    
    fn to_base_prime_field_elements(&self) -> Self::BasePrimeFieldIter {
        todo!()
    }
    
    fn from_base_prime_field_elems(elems: &[Self::BasePrimeField]) -> Option<Self> {
        todo!()
    }
    
    fn from_base_prime_field(elem: Self::BasePrimeField) -> Self {
        todo!()
    }
    
    fn double_in_place(&mut self) -> &mut Self {
        todo!()
    }
    
    fn neg_in_place(&mut self) -> &mut Self {
        todo!()
    }
    
    fn from_random_bytes_with_flags<F: Flags>(bytes: &[u8]) -> Option<(Self, F)> {
        todo!()
    }
    
    fn legendre(&self) -> ark_ff::LegendreSymbol {
        todo!()
    }
    
    fn square(&self) -> Self {
        todo!()
    }
    
    fn square_in_place(&mut self) -> &mut Self {
        todo!()
    }
    
    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        todo!()
    }
    
    fn frobenius_map_in_place(&mut self, power: usize) {
        todo!()
    }
    
    fn characteristic() -> &'static [u64] {
        &[]
    }
    
    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        std::unimplemented!()
    }
    
    fn sqrt(&self) -> Option<Self> {
        std::unimplemented!();
    }
    
    fn sqrt_in_place(&mut self) -> Option<&mut Self> {
        std::unimplemented!();
    }
    
    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }
    
    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        *self
    }
    
    fn pow_with_table<S: AsRef<[u64]>>(powers_of_2: &[Self], exp: S) -> Option<Self> {
        std::unimplemented!()
    }

}
