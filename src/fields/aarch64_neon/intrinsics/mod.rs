use ark_std::{
    arch::aarch64::{
        uint32x4_t, vaddq_u32, vandq_u32, vcgeq_u32, vdupq_n_u32, vld1q_u32, vminq_u32, vmlsq_u32,
        vmulq_u32, vqdmulhq_s32, vreinterpretq_s32_u32, vreinterpretq_u32_s32, vst1q_u32,
        vsubq_u32,
    },
    mem::transmute,
};

use crate::fields::m31::reduce_sum_naive;

#[inline(always)]
fn sum_vectors(v0: &mut uint32x4_t, v1: &uint32x4_t, packed_modulus: &uint32x4_t) {
    let raw_sum = unsafe { vaddq_u32(*v0, *v1) };
    let gte_mask = unsafe { vcgeq_u32(raw_sum, *packed_modulus) };
    *v0 = unsafe { vsubq_u32(raw_sum, vandq_u32(*packed_modulus, gte_mask)) };
    // an alternative to the above three lines is this, you can experiment to see which is more performant
    // let sum1 = vaddq_u32(*v0, *v1);
    // let sum2 = vsubq_u32(sum1, *packed_modulus);
    // *v0 = vminq_u32(sum1, vandq_u32(*packed_modulus, sum2));
}

pub fn reduce_sum_32_bit_modulus(values: &[u32], modulus: u32) -> u32 {
    let modulus: uint32x4_t = unsafe { transmute::<[u32; 4], uint32x4_t>([modulus; 4]) };
    let mut sums: uint32x4_t = unsafe { vdupq_n_u32(0) };

    // TODO (z-tech): This should be unrolled, you have to figure out how much unrolling is the sweet spot (try 16, 32, ...)
    for step in (0..values.len()).step_by(4) {
        let v: uint32x4_t = unsafe { vld1q_u32(values.as_ptr().add(step)) };
        sum_vectors(&mut sums, &v, &modulus);
    }

    let arr: [u32; 4] = unsafe { transmute(sums) };
    reduce_sum_naive(&arr)
}

pub fn scalar_mult_32_bit_modulus(values: &mut [u32], scalar: u32, modulus: u32) {
    let packed_modulus: uint32x4_t = unsafe { transmute::<[u32; 4], uint32x4_t>([modulus; 4]) };
    let packed_scalar: uint32x4_t = unsafe { transmute::<[u32; 4], uint32x4_t>([scalar; 4]) };
    for step in (0..values.len()).step_by(4) {
        unsafe {
            let lhs = vld1q_u32(values.as_ptr().add(step));
            let upper = vreinterpretq_u32_s32(vqdmulhq_s32(
                vreinterpretq_s32_u32(lhs),
                vreinterpretq_s32_u32(packed_scalar),
            ));
            let lower = vmulq_u32(lhs, packed_scalar);
            let t = vmlsq_u32(lower, upper, packed_modulus);
            let res = vminq_u32(
                vmlsq_u32(lower, upper, packed_modulus),
                vsubq_u32(t, packed_modulus),
            );
            vst1q_u32(values.as_mut_ptr().add(step), res);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fields::{
        aarch64_neon::reduce_sum_32_bit_modulus,
        m31::{M31, M31_MODULUS},
    };
    use ark_ff::Zero;
    use ark_std::test_rng;

    #[test]
    fn reduce_sum_correctness() {
        fn reduce_sum_sanity(vec: &[M31]) -> M31 {
            M31::from(vec.iter().fold(M31::zero(), |acc, &x| (acc + x)))
        }

        let mut rng = test_rng();
        let random_field_values: Vec<M31> = (0..1 << 13).map(|_| M31::rand(&mut rng)).collect();
        let random_field_values_u32: Vec<u32> =
            random_field_values.iter().map(|m| m.to_u32()).collect();
        let exp = reduce_sum_sanity(&random_field_values);
        assert_eq!(
            exp,
            M31::from(reduce_sum_32_bit_modulus(
                &random_field_values_u32,
                M31_MODULUS
            ))
        );
    }
}
