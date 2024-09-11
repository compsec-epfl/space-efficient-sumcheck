use std::{self, fmt::{self, Display, Formatter}, io::{Read, Write}, num::ParseIntError, str::FromStr};

use ark_ff::{BigInt, FftField, Field, Fp, FpConfig, One, PrimeField, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerializeWithFlags, CanonicalSerialize, EmptyFlags, Flags, SerializationError};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use num_bigint::BigUint;
use zeroize::Zeroize;
use ark_ff::biginteger::BigInteger256;

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

impl Zeroize for Field32 {
    fn zeroize(&mut self) {
        // Overwrite the sensitive fields with zero
        // self.value.zeroize();
        // // Optionally, you can zero out modulus as well if it's considered sensitive
        // self.modulus.zeroize();
    }
}

impl Distribution<Field32> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Field32 {
        let modulus = 97; // Example modulus, you can customize this
        let value = rng.gen_range(0..modulus);
        Field32::new(value, modulus)
    }
}

impl From<Field32> for BigInt<4> {
    fn from(field: Field32) -> BigInt<4> {
        // Place the value in the first limb, and leave the rest as zero
        BigInt::<4>([field.value as u64, 0, 0, 0])
    }
}

impl From<BigUint> for Field32 {
    fn from(biguint: BigUint) -> Self {
        // Convert BigUint to u32, ensuring it fits within the modulus
        let modulus = 97; // Example modulus, should match the Field32's modulus

        // Reduce the BigUint value modulo the field's modulus
        let reduced_value = biguint % BigUint::from(modulus);

        // Convert reduced BigUint to u32 (check for overflow)
        let value = 1;

        Field32::new(value, modulus)
    }
}

impl From<BigInteger256> for Field32 {
    fn from(bigint: BigInteger256) -> Self {
        let modulus = 97; // Example modulus, should match your field's modulus

        // Convert BigInteger256 to a u64
        let bigint_u64 = bigint.0[0];

        // Reduce the BigInteger256 value modulo the field's modulus
        let reduced_value = bigint_u64 % (modulus as u64);

        // Convert reduced value to u32
        let value = reduced_value as u32;

        Field32::new(value, modulus)
    }
}

impl FromStr for Field32 {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Define modulus for the field
        let modulus = 97; // Example modulus, should match your field's modulus

        // Parse the string to a u32
        let value = u32::from_str(s)?;

        // Reduce the parsed value modulo the field's modulus
        let reduced_value = value % modulus;

        Ok(Field32::new(reduced_value, modulus))
    }
}

impl Display for Field32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", 1, 1)
        }
     }

impl PrimeField for Field32 {
    type BigInt = BigInteger256;

    const MODULUS: Self::BigInt = BigInteger256::one();

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger256::one();

    const MODULUS_BIT_SIZE: u32 = 1;

    const TRACE: Self::BigInt = BigInteger256::one();

    const TRACE_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger256::one();

    fn from_bigint(repr: Self::BigInt) -> Option<Self> {
        todo!()
    }

    fn into_bigint(self) -> Self::BigInt {
        todo!()
    }
    
    fn from_be_bytes_mod_order(bytes: &[u8]) -> Self {
        Self { value: 0, modulus: FIELD_32_MODULUS }
    }
    
    fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        Self { value: 0, modulus: FIELD_32_MODULUS }
    }
}

impl FftField for Field32 {
    const GENERATOR: Self = Field32 { value: 5, modulus: 97 };

    const TWO_ADICITY: u32 = 1;

    const TWO_ADIC_ROOT_OF_UNITY: Self = Field32 { value: 5, modulus: 97 };
    
    const SMALL_SUBGROUP_BASE: Option<u32> = None;
    
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;
    
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = None;
    
    fn get_root_of_unity(n: u64) -> Option<Self> {
        None
    }
}

impl CanonicalDeserializeWithFlags for Field32 {
    #[inline]
    fn deserialize_with_flags<R: Read, F: Flags>(
        mut reader: R,
    ) -> Result<(Self, F), SerializationError> {
        Ok((Self { value: 1, modulus: 1}, F::from_u8(1).unwrap()))
    }
}

impl CanonicalSerializeWithFlags for Field32 {
    #[inline]
    fn serialize_with_flags<W: Write, F: Flags>(
        &self,
        mut writer: W,
        flags: F,
    ) -> Result<(), SerializationError> {
        Ok(())
    }

    #[inline]
    fn serialized_size_with_flags<F: Flags>(&self) -> usize {
        1
    }
}

impl Default for Field32 {
    fn default() -> Self {
        // Define what the default value should be
        // Here, we use `0` as the default value and `1` as the modulus.
        Field32::new(0, 1)
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
