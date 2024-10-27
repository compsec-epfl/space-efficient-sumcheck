use ark_ff::{FftField, Field, One, Zero};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Flags, SerializationError,
};
use ark_std::rand::{distributions::Standard, prelude::Distribution, Rng};
use zeroize::Zeroize;

use std::simd::cmp::SimdPartialOrd;
use std::simd::{u32x4, u32x64, u64x64, Simd};
use std::{
    fmt::{self, Display, Formatter},
    io::{Read, Write},
};

pub mod fft_field;
pub mod field;
pub mod ops;
pub mod prime_field;
pub mod transmute;

pub const BB_MODULUS: u32 = 0x78000001;
pub const BB_MODULUS_U64: u64 = BB_MODULUS as u64;
pub const BB_MODULUS_U128: u128 = BB_MODULUS as u128;
pub const BB_MODULUS_USIZE: usize = BB_MODULUS as usize;
pub const BB_MODULUS_I32: i32 = BB_MODULUS as i32;
pub const BB_MODULUS_I64: i64 = BB_MODULUS as i64;

const N: u64 = 2013265921; // Baby Bear Prime
                           // const R: u64 = 4294967296; // 2^32 which is > N
const R_MINUS_ONE: u64 = 4294967295;
const R_PRIME: u64 = 943718400; // Chosen s.t. RR′ ≡ 1 (mod N)
const N_PRIME: u64 = 2013265919; // Chosen s.t. NN′ ≡ -1 (mod R)

const sixty_fourth: u32 = (BB_MODULUS >> 6) + 1;
const thirty_second: u32 = (BB_MODULUS >> 5) + 1;
const sixteenth: u32 = (BB_MODULUS >> 4) + 1;
const eighth: u32 = (BB_MODULUS >> 3) + 1;
const quarter: u32 = (BB_MODULUS >> 2) + 1;
const half: u32 = (BB_MODULUS >> 1) + 1;
const eighth_sixteenth: u32 = eighth + sixteenth;
const quarter_eighth: u32 = quarter + eighth;
const quarter_sixteenth: u32 = quarter + sixteenth;
const quarter_eighth_sixteenth: u32 = quarter + eighth + sixteenth;
const half_quarter: u32 = half + quarter;
const half_eighth: u32 = half + eighth;
const half_sixteenth: u32 = half + sixteenth;
const half_eighth_sixteenth: u32 = half + eighth + sixteenth;
const half_quarter_eighth: u32 = half + quarter + eighth;
const half_quarter_sixteenth: u32 = half + quarter + sixteenth;
const half_quarter_eighth_sixteenth: u32 = half + quarter + eighth + sixteenth;

const sixty_fourth_thirty_second: u32 = sixty_fourth + thirty_second;
const sixteenth_sixty_fourth: u32 = sixteenth + sixty_fourth;

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord,
    Hash,
    CanonicalDeserialize,
    CanonicalSerialize,
)]
pub struct BabyBear {
    mod_value: u32,
}

const LANES: usize = 64;

impl BabyBear {
    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = self.clone();
        for _ in 0..power_log {
            res = res.square();
        }
        res
    }

    pub fn reduce_sum(vec: &[u32]) -> Self {
        let sum: u32 = vec.iter().fold(0, |acc, &x| {
            let tmp = acc + x;
            if tmp < BB_MODULUS {
                return tmp;
            } else {
                return tmp - BB_MODULUS;
            }
        });
        Self { mod_value: sum }
    }

    pub fn reduce_sum_packed(values: &[u32]) -> Self {
        assert!(values.len() % LANES == 0);
        let packed_modulus: Simd<u32, LANES> = u32x64::splat(BB_MODULUS);
        let mut packed_sums: Simd<u32, LANES> = u32x64::splat(0);
        for i in (0..values.len()).step_by(64) {
            let tmp_packed_sums = packed_sums + u32x64::from_slice(&values[i..i + 64]);
            let is_mod_needed = tmp_packed_sums.simd_ge(packed_modulus);
            packed_sums = is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
        }
        Self::reduce_sum(&packed_sums.to_array())
    }

    pub fn reduce_sum_like_thing_packed(value: u32, power_of_2: u32) -> Self {
        let packed_modulus: Simd<u32, LANES> = u32x64::splat(BB_MODULUS);
        let packed_value: Simd<u32, LANES> = u32x64::splat(value);
        let mut packed_sums: Simd<u32, LANES> = u32x64::splat(0);
        for _ in (0..2_i32.pow(power_of_2)).step_by(64) {
            let tmp_packed_sums = packed_sums + packed_value;
            let is_mod_needed = tmp_packed_sums.simd_ge(packed_modulus);
            packed_sums = is_mod_needed.select(tmp_packed_sums - packed_modulus, tmp_packed_sums);
        }
        Self::reduce_sum(&packed_sums.to_array())
    }

    pub fn to_mont(source: u32) -> u64 {
        ((source as u64) << 32) % N
    }

    pub fn from_mont(source: u64) -> u32 {
        ((source * R_PRIME) % N) as u32
    }

    pub fn mont_mult(a: u64, b: u64) -> u64 {
        // calculate product in montgomery form
        let prod = a * b;
        // reduce step 1
        let m = ((prod & R_MINUS_ONE) * N_PRIME) & R_MINUS_ONE;
        // reduce steb 2
        let t = (prod + m * BB_MODULUS_U64) >> 32;
        // reduce step 3 and leave mont space
        t - match t >= BB_MODULUS_U64 {
            true => BB_MODULUS_U64,
            false => 0, // always subtract, timing attack possible
        }
    }

    // fn mul_u32_parts(a_hi: u32, a_lo: u32, b_hi: u32, b_lo: u32) -> (u32, u32) {
    //     // Step 1: Multiply the parts
    //     let lo_lo = a_lo * b_lo;
    //     let hi_lo = a_hi * b_lo;
    //     let lo_hi = a_lo * b_hi;
    //     let hi_hi = a_hi * b_hi;

    //     // Step 2: Combine the low and high parts carefully
    //     let middle = hi_lo + lo_hi; // Combine cross terms
    //     let low = lo_lo + (middle << 16); // Add middle term, shifted
    //     let carry = if low < lo_lo { 1 } else { 0 }; // Check for overflow

    //     // Add high part, handle carries
    //     let high = hi_hi + (middle >> 16) + carry;

    //     (high, low)
    // }

    fn mul_hi_hi_sanity(a_hi: u32, b_hi: u32) -> u64 {
        (a_hi as u64 * b_hi as u64) << 32
    }

    pub fn batch_mult_normal(values: &mut [u32], multipland: u32) {
        for elem in values.iter_mut() {
            *elem = ((*elem as u64 * multipland as u64) % BB_MODULUS_U64) as u32;
        }
    }

    // pub fn batch_mult_2(values: &mut [u32], multipland: u32) {
    //     let multiplicand_hi = multipland >> 16;
    //     let multiplicand_lo = multipland & 0xFFFF;
    //     for elem in values.iter_mut() {
    //         let multiplicand_parts: Simd<u32, 4> = u32x4::from_slice(&[multiplicand_hi, multiplicand_lo, multiplicand_hi, multiplicand_lo]);
    //         let value_parts: Simd<u32, 4> = u32x4::from_slice(&[*elem >> 16, *elem >> 16, *elem & 0xFFFF, *elem & 0xFFFF]);
    //         let product = multiplicand_parts * value_parts;
    //         *elem = ((*elem as u64 * multipland as u64) % BB_MODULUS_U64) as u32;
    //     }
    // }

    pub fn batch_mult_mont(values: &mut [u32], multipland: u32) {
        let multiplicand = Self::to_mont(multipland);
        for elem in values.iter_mut() {
            *elem = Self::from_mont(Self::mont_mult(Self::to_mont(*elem), multiplicand));
        }
    }

    pub fn batch_mult_parts_2(values: &mut [u32], multipland: u32) {
        fn extra_quick_mod(a: u32) -> u32 {
            if a >= BB_MODULUS {
                return a - BB_MODULUS;
            } else {
                return a;
            }
        }
        fn quick_mod(a: u32) -> u32 {
            if a >= 4026531842 {
                return a - 4026531842;
            } else if a >= BB_MODULUS {
                return a - BB_MODULUS;
            } else {
                return a;
            }
        }
        fn sub_amount(hi_hi: u32) -> u64 {
            let sub = if hi_hi < sixteenth {
                0
            } else if hi_hi < eighth {
                BB_MODULUS_U64
            } else if hi_hi < eighth_sixteenth {
                2 * BB_MODULUS_U64
            } else if hi_hi < quarter {
                3 * BB_MODULUS_U64
            } else if hi_hi < quarter_sixteenth {
                4 * BB_MODULUS_U64
            } else if hi_hi < quarter_eighth {
                5 * BB_MODULUS_U64
            } else if hi_hi < quarter_eighth_sixteenth {
                6 * BB_MODULUS_U64
            } else if hi_hi < half {
                7 * BB_MODULUS_U64
            } else if hi_hi < half_sixteenth {
                8 * BB_MODULUS_U64
            } else if hi_hi < half_eighth {
                9 * BB_MODULUS_U64
            } else if hi_hi < half_eighth_sixteenth {
                10 * BB_MODULUS_U64
            } else if hi_hi < half_quarter {
                11 * BB_MODULUS_U64
            } else if hi_hi < half_quarter_sixteenth {
                12 * BB_MODULUS_U64
            } else if hi_hi < half_quarter_eighth {
                13 * BB_MODULUS_U64
            } else if hi_hi < half_quarter_eighth_sixteenth {
                14 * BB_MODULUS_U64
            } else {
                15 * BB_MODULUS_U64
            };
            sub
        }
        fn calculate_b(a: u32) -> u64 {
            ((2 * a) + 1 + 2 * ((a - 8) / 15) + (((a - 8) % 15) / 8)) as u64 * BB_MODULUS_U64
        }
        let multiplicand_hi = multipland >> 16;
        let multiplicand_lo = multipland & 0xFFFF;
        for elem in values.iter_mut() {
            let hi = *elem >> 16;
            let lo = *elem & 0xFFFF;

            // no overflow bc u16 * u16
            let mut hi_hi = hi * multiplicand_hi;
            let mut hi_lo = hi * multiplicand_lo;
            let mut lo_hi = lo * multiplicand_hi;
            let lo_lo = lo * multiplicand_lo;

            // println!("hi_hi is: {}", hi_hi);
            // assert_eq!(sub_amount(hi_hi), calculate_b(hi_hi));
            // let exp = ((hi_hi as u64) << 32) % BB_MODULUS_U64;
            // let tmp = ((hi_hi as u64) << 32) - calculate_b(hi_hi);

            // hi_hi = quick_mod(hi_hi);
            // do this eight times
            // let s = hi_hi as u64 / (BB_MODULUS_U64 >> 7); // there's only 128 of these, can be precomputed
            // hi_hi = (((hi_hi as u64) << 32) - (BB_MODULUS_U64 * s)) as u32;
            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;
            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;

            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;
            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;

            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;
            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;

            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;
            // hi_hi = (((hi_hi as u64) << 4) - sub_amount(hi_hi)) as u32;

            // assert_eq!(exp, hi_hi as u64);
            // assert_eq!(tmp, hi_hi as u64);

            hi_hi = (((hi_hi as u64) << 32) % BB_MODULUS_U64) as u32; // 2^32 mod n is 268435454
            hi_lo = (((hi_lo as u64) << 16) % BB_MODULUS_U64) as u32;
            lo_hi = (((lo_hi as u64) << 16) % BB_MODULUS_U64) as u32;

            let mut result = extra_quick_mod(hi_hi + hi_lo);
            result = extra_quick_mod(result + lo_hi);
            result = extra_quick_mod(result + lo_lo);

            *elem = result;
        }
    }

    // pub packed_quick_mod(mut a: u32x64) {
    //     let packed_sixty_fourth  = u32x64::splat(sixty_fourth);
    //     let packed_thirty_second  = u32x64::splat(thirty_second);
    //     let packed_sixteenth  = u32x64::splat(sixteenth);
    //     let packed_eighth  = u32x64::splat(eighth);
    //     let packed_quarter  = u32x64::splat(quarter);
    //     let packed_half  = u32x64::splat(half);
    //     let packed_eighth_sixteenth  = u32x64::splat(eighth_sixteenth);
    //     let packed_quarter_eighth  = u32x64::splat(quarter_eighth);
    //     let packed_quarter_sixteenth  = u32x64::splat(quarter_sixteenth);
    //     let packed_quarter_eighth_sixteenth  = u32x64::splat(quarter_eighth_sixteenth);
    //     let packed_half_quarter  = u32x64::splat(half_quarter);
    //     let packed_half_eighth  = u32x64::splat(half_eighth);
    //     let packed_half_sixteenth  = u32x64::splat(half_sixteenth);
    //     let packed_eighth_sixteenth  = u32x64::splat(eighth_sixteenth);
    //     let packed_quarter_eighth  = u32x64::splat(quarter_eighth);
    //     let packed_quarter_sixteenth  = u32x64::splat(quarter_sixteenth);
    //     let packed_quarter_eighth_sixteenth  = u32x64::splat(quarter_eighth_sixteenth);

    // }
    pub fn batch_mult_parts(values: &mut [u32], multiplicand: u32) {
        assert!(values.len() % LANES == 0);
        let multiplicand_lo: Simd<u32, LANES> = u32x64::splat(multiplicand & 0xFFFF);
        let multiplicand_hi: Simd<u32, LANES> = u32x64::splat(multiplicand >> 16);
        let modulus: Simd<u32, LANES> = u32x64::splat(BB_MODULUS);
        for i in (0..values.len()).step_by(64) {
            let lo = u32x64::from_slice(&values[i..i + 64]) & u32x64::splat(0xFFFF);
            let hi = u32x64::from_slice(&values[i..i + 64]) >> 16;
            let mut hi_hi = hi * multiplicand_hi;
            let mut hi_lo = hi * multiplicand_lo;
            let mut lo_hi = lo * multiplicand_hi;
            let lo_lo = lo * multiplicand_lo;

            // ensure the products are in the field
            for _ in 0..2 {
                hi_hi = hi_hi.simd_ge(modulus).select(hi_hi - modulus, hi_hi);
                hi_lo = hi_lo.simd_ge(modulus).select(hi_lo - modulus, hi_lo);
                lo_hi = lo_hi.simd_ge(modulus).select(lo_hi - modulus, lo_hi);
            }

            // then 32 shifts and 32 mod checks
            for i in 0..32 {
                hi_hi = hi_hi << 1;
                hi_hi = hi_hi.simd_ge(modulus).select(hi_hi - modulus, hi_hi);
                // only 16 for hi_lo and lo_hi
                if i < 16 {
                    hi_lo = hi_lo << 1;
                    hi_lo = hi_lo.simd_ge(modulus).select(hi_lo - modulus, hi_lo);
                    lo_hi = lo_hi << 1;
                    lo_hi = lo_hi.simd_ge(modulus).select(lo_hi - modulus, lo_hi);
                }
            }

            // combine
            hi_hi = hi_hi + hi_lo;
            hi_hi = hi_hi.simd_ge(modulus).select(hi_hi - modulus, hi_hi);
            hi_hi = hi_hi + lo_hi;
            hi_hi = hi_hi.simd_ge(modulus).select(hi_hi - modulus, hi_hi);
            hi_hi = hi_hi + lo_lo;
            hi_hi = hi_hi.simd_ge(modulus).select(hi_hi - modulus, hi_hi);

            // write back in
            values[i..i + 64].copy_from_slice(&hi_hi.to_array());
        }
    }

    pub fn batch_mult_normal_packed(values: &mut [u32], multipland: u32) {
        fn convert_u32_to_u64(slice: &[u32]) -> Vec<u64> {
            slice.iter().map(|&num| num as u64).collect()
        }
        assert!(values.len() % LANES == 0);
        let packed_multiplicand: Simd<u64, LANES> = u64x64::splat(multipland as u64);
        let packed_modulus: Simd<u64, LANES> = u64x64::splat(BB_MODULUS as u64);
        let mut packed_product: Simd<u64, LANES>;
        for i in (0..values.len()).step_by(64) {
            let slice: &[u64] = &convert_u32_to_u64(&mut values[i..i + 64]);
            packed_product = u64x64::from_slice(slice) * packed_multiplicand;
            packed_product = packed_product % packed_modulus;
            let products: &[u32; 64] = &packed_product
                .to_array()
                .map(|product: u64| -> u32 { product as u32 });
            values[i..i + 64].copy_from_slice(products);
        }
    }
}

impl Zero for BabyBear {
    fn zero() -> Self {
        BabyBear::from(0_u8)
    }
    fn is_zero(&self) -> bool {
        self.mod_value == 0
    }
}

impl One for BabyBear {
    fn one() -> Self {
        BabyBear::from(1_u8)
    }
    fn is_one(&self) -> bool {
        self.to_u32() == 1_u32
    }
}

impl Zeroize for BabyBear {
    fn zeroize(&mut self) {
        todo!()
    }
}

impl Distribution<BabyBear> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BabyBear {
        let value = rng.gen_range(0..BB_MODULUS);
        BabyBear::from(value)
    }
}

impl Display for BabyBear {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.mod_value, f)
    }
}

impl CanonicalDeserializeWithFlags for BabyBear {
    #[inline]
    fn deserialize_with_flags<R: Read, F: Flags>(
        _reader: R,
    ) -> Result<(Self, F), SerializationError> {
        Ok((Self { mod_value: 1 }, F::from_u8(1).unwrap()))
    }
}

impl CanonicalSerializeWithFlags for BabyBear {
    #[inline]
    fn serialize_with_flags<W: Write, F: Flags>(
        &self,
        _writer: W,
        _flags: F,
    ) -> Result<(), SerializationError> {
        Ok(())
    }

    #[inline]
    fn serialized_size_with_flags<F: Flags>(&self) -> usize {
        1
    }
}

impl Default for BabyBear {
    fn default() -> Self {
        BabyBear::from(1_u8)
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::{Field, UniformRand};
    use ark_std::test_rng;

    use crate::fields::baby_bear::{BabyBear, BB_MODULUS, BB_MODULUS_U64};

    // use super::BB_MODULUS_U64;

    // fn to_parts(value: u32) -> (u32, u32) {
    //     (value >> 16, value & 0xFFFF)
    // }

    // fn barrett_reduction(x: u128, m: u128, mu: u128) -> u128 {
    //     let k = (m.leading_zeros() - 1) as u32; // number of bits in m
    //     let b_k = 1 << k; // b^k
    //     let q1 = x >> (k - 1);
    //     let q2 = q1.wrapping_mul(mu);
    //     let q3 = q2 >> (k + 1);
    //     let r1 = x % b_k;
    //     let r2 = (q3.wrapping_mul(m)) % b_k;
    //     let mut r = r1.wrapping_sub(r2);

    //     if (r as i128) < 0 {
    //         r = r.wrapping_add(b_k);
    //     }

    //     while r >= m {
    //         r = r.wrapping_sub(m);
    //     }

    //     r
    // }

    // fn montgomery_multiplication(a: u128, b: u128, m: u128, r_inv: u128, n: u32) -> u128 {
    //     let t = a.wrapping_mul(b);
    //     let m_prime = r_inv.wrapping_mul(t & ((1 << n) - 1)) & ((1 << n) - 1);
    //     let u = (t + m_prime.wrapping_mul(m)) >> n;

    //     if u >= m {
    //         u.wrapping_sub(m)
    //     } else {
    //         u
    //     }
    // }

    #[test]
    fn inverse_correctness() {
        let a = BabyBear::from(2);
        assert_eq!(BabyBear::from(1006632961), a.inverse().unwrap());
    }

    #[test]
    fn reduce_sum_correctness() {
        fn reduce_sum_sanity(vec: &[u32]) -> BabyBear {
            BabyBear::from(vec.iter().fold(0, |acc, &x| (acc + x) % BB_MODULUS))
        }

        let mut rng = test_rng();
        let random_field_values: Vec<u32> = (0..1 << 13)
            .map(|_| BabyBear::rand(&mut rng).to_u32())
            .collect();
        let exp = reduce_sum_sanity(&random_field_values);
        assert_eq!(exp, BabyBear::reduce_sum(&random_field_values));
        assert_eq!(exp, BabyBear::reduce_sum_packed(&random_field_values));
    }

    #[test]
    fn batch_mult_correctness() {
        fn sub_amount(mut a: i32, b: i32, shift: u32) -> i32 {
            // if reps > 0 {
            //     if a  < b / 2 {
            //         return sub_amount(a - (a - b).abs() + b, b, reps - 1);
            //     } else {
            //         return sub_amount(a - (a - b).abs(), b, reps - 1);
            //     }
            // }
            // a as u32
            let res = (((a as u64) << shift) % b as u64) as i32;
            // res = a - b;
            // res - a = - b
            // b = a - res
            println!("a {}, res {}", a, res);
            a - res
        }
        // fn divide_by_15(x: u32) -> u32 {
        //     let m: u32 = 0x88888889; // Magic number
        //     (x.wrapping_mul(m) >> 34) as u32
        // }
        // fn mod_by_15(x: u32) -> u32 {
        //     let m: u32 = 0x88888889; // Precomputed magic number for division by 15
        //     let q = (x.wrapping_mul(m) >> 34) as u32; // Approximate division by 15
        //     x - q * 15 // Compute the modulus by subtracting the result
        // }
        let mut rng = test_rng();
        let multiplicand = 999999_u32;
        // let mut exp: Vec<u32> = (0..1 << 13)
        //     .map(|_| BabyBear::rand(&mut rng).to_u32())
        //     .collect();
        let mut exp: Vec<u32> = (0..(2 << 13)).collect();

        let mut last_a_causing_problem = 8;
        let mut last_d = 14;
        for a in &exp {
            let res = ((*a as u64) << 2) % BB_MODULUS_U64;
            // let d = (((*a as u64) << 32) - res) / BB_MODULUS_U64;
            // let num_segments = 1 <<  4;
            // let seg_size = BB_MODULUS / num_segments;
            // println!("{}", seg_size);
            // let tmp = (a / seg_size);
            // // println!("{},{}", *a, d);
            // assert_eq!(d, tmp as u64);
            // assert_eq!(
            //     d as u32,
            //     2 * *a
            //         + (std::cmp::max(
            //             3,
            //             if a.leading_zeros() == 0 {
            //                 0
            //             } else {
            //                 31 - a.leading_zeros()
            //             }
            //         ) - 3)
            // );
            // 4 = (5 << 5) - 13x
            println!("{},{}", res, sub_amount(*a as i32, BB_MODULUS as i32, 4));
            // assert_eq!(res as u32, sub_amount(*a as i32, BB_MODULUS as i32, 2), "hello {}", a);
            // assert_eq!(a / 15, divide_by_15(*a));
            // assert_eq!(d, ((2*a) + ((a - 8) / 15) + ((a % 15) / 8)) as u64);
            // assert_eq!(d, ((2*a) + 1+2*((a - 8) / 15) + ((((a-8) % 15)) / 8)) as u64);
            // assert_eq!(d, (2*a +2*(a % 15)-(a % 15)%7) as u64);
            // assert_eq!(res, ((a as u64) << 32) - (d * BB_MODULUS_U64));
            // if (d - last_d > 2) {
            //     println!("at index {}, delta is {}, value is {}, distance between was {}", *a, d - last_d, d, *a - last_a_causing_problem);
            //     last_a_causing_problem = *a;
            // }
            // last_d = d;
        }
        // println!("{:?}", exp);
        let mut mont: Vec<u32> = exp.clone();
        let mut parts: Vec<u32> = exp.clone();
        let mut normal_packed: Vec<u32> = exp.clone();

        // normal
        BabyBear::batch_mult_normal(&mut exp, multiplicand);

        // mont
        BabyBear::batch_mult_mont(&mut mont, multiplicand);
        assert_eq!(exp, mont,);

        // parts
        BabyBear::batch_mult_parts_2(&mut parts, multiplicand);
        // for (i, value) in exp.into_iter().enumerate() {
        //     println!("index: {}", i);
        //     assert_eq!(value, parts[i]);
        // }
        // assert_eq!(exp, parts);

        // normal + packed
        // BabyBear::batch_mult_normal_packed(&mut normal_packed, multiplicand);
        // assert_eq!(exp, normal_packed);
    }
}
