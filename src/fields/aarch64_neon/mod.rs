use ark_std::{arch::aarch64, cmp, mem::transmute};

use crate::fields::m31::M31_MODULUS;

#[inline]
fn sum_vectors(
    v0: &mut aarch64::uint32x4_t,
    v1: &aarch64::uint32x4_t,
    packed_modulus: &aarch64::uint32x4_t,
) {
    let raw_sum = unsafe { aarch64::vaddq_u32(*v0, *v1) };
    let gte_mask = unsafe { aarch64::vcgeq_u32(raw_sum, *packed_modulus) };
    *v0 = unsafe { aarch64::vsubq_u32(raw_sum, aarch64::vandq_u32(*packed_modulus, gte_mask)) };
    // an alternative is this (it seems a touch slower):
    // let sum1 = aarch64::vaddq_u32(*v0, *v1);
    // let sum2 = aarch64::vsubq_u32(sum1, *packed_modulus);
    // *v0 = aarch64::vminq_u32(sum1, aarch64::vandq_u32(*packed_modulus, sum2));
}

#[inline]
fn sum_lanes(lanes: &aarch64::uint32x4_t) -> u32 {
    let reduced_sum: u32 = [
        unsafe { aarch64::vgetq_lane_u32(*lanes, 0) },
        unsafe { aarch64::vgetq_lane_u32(*lanes, 1) },
        unsafe { aarch64::vgetq_lane_u32(*lanes, 2) },
        unsafe { aarch64::vgetq_lane_u32(*lanes, 3) },
    ]
    .iter()
    .fold(0, |acc, &x| {
        let sum1 = acc + x;
        let sum2 = sum1.wrapping_sub(M31_MODULUS);
        return cmp::min(sum1, sum2);
    });
    reduced_sum
}

pub fn reduce_sum_32_bit_modulus(values: &[u32], modulus: u32) -> u32 {
    let packed_modulus: aarch64::uint32x4_t =
        unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([modulus; 4]) };
    let mut sums: aarch64::uint32x4_t = unsafe { aarch64::vdupq_n_u32(0) };

    // Note: it's a big ugly function bc it must be unrolled to fill up available registers
    for step in (0..values.len()).step_by(64) {
        // sum the first 8 vectors into v0, v2, v4, v6
        let mut v0 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step)) };
        let v1 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 4)) };
        sum_vectors(&mut v0, &v1, &packed_modulus);
        let mut v2 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 8)) };
        let v3 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 12)) };
        sum_vectors(&mut v2, &v3, &packed_modulus);
        let mut v4 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 16)) };
        let v5 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 20)) };
        sum_vectors(&mut v4, &v5, &packed_modulus);
        let mut v6 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 24)) };
        let v7 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 28)) };
        sum_vectors(&mut v6, &v7, &packed_modulus);

        // sum the next 8 vectors into v8, v10, v12, v14
        let mut v8 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 32)) };
        let v9 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 36)) };
        sum_vectors(&mut v8, &v9, &packed_modulus);
        let mut v10 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 40)) };
        let v11 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 44)) };
        sum_vectors(&mut v10, &v11, &packed_modulus);
        let mut v12 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 48)) };
        let v13 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 52)) };
        sum_vectors(&mut v12, &v13, &packed_modulus);
        let mut v14 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 56)) };
        let v15 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 60)) };
        sum_vectors(&mut v14, &v15, &packed_modulus);

        // sum v2, v4, v6 into v0
        sum_vectors(&mut v0, &v2, &packed_modulus);
        sum_vectors(&mut v0, &v4, &packed_modulus);
        sum_vectors(&mut v0, &v6, &packed_modulus);

        // sum v10, v12, v14 into v8
        sum_vectors(&mut v8, &v10, &packed_modulus);
        sum_vectors(&mut v8, &v12, &packed_modulus);
        sum_vectors(&mut v8, &v14, &packed_modulus);

        // sum the accumulated sums into sums
        sum_vectors(&mut sums, &v0, &packed_modulus);
        sum_vectors(&mut sums, &v8, &packed_modulus);
    }
    sum_lanes(&sums)
}

pub fn scalar_mult_32_bit_modulus(values: &mut [u32], scalar: u32, modulus: u32) {
    let packed_modulus: aarch64::uint32x4_t =
        unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([modulus; 4]) };
    let packed_scalar: aarch64::uint32x4_t =
        unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([scalar; 4]) };
    for step in (0..values.len()).step_by(4) {
        // can I unroll this?
        unsafe {
            let lhs = aarch64::vld1q_u32(values.as_ptr().add(step));
            // let mut tmp: [u32; 4] = [
            //     unsafe { aarch64::vgetq_lane_u32(lhs, 0) },
            //     unsafe { aarch64::vgetq_lane_u32(lhs, 1) },
            //     unsafe { aarch64::vgetq_lane_u32(lhs, 2) },
            //     unsafe { aarch64::vgetq_lane_u32(lhs, 3) },
            // ];
            // println!(
            //     "{:?},{:?},{:?},{:?}",
            //     values[step],
            //     values[step + 1],
            //     values[step + 2],
            //     values[step + 3]
            // );
            // assert_eq!(tmp[0], 1695424478);
            let mut hi = aarch64::vandq_u32(
                aarch64::vreinterpretq_u32_s32(aarch64::vqdmulhq_s32(
                    aarch64::vreinterpretq_s32_u32(lhs),
                    aarch64::vreinterpretq_s32_u32(packed_scalar),
                )),
                aarch64::vdupq_n_u32(0xFFFFFFFE),
            );

            // let mut hi_values: [u32; 4] = [
            //     unsafe { aarch64::vgetq_lane_u32(hi, 0) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 1) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 2) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 3) },
            // ];
            // // println!("hi_values[0]: {:?}", hi_values[0]);
            // // assert_eq!(
            // //     hi_values[0],
            // //     (((values[0] as u64) * (scalar as u64)) >> 32) as u32
            // // );

            // // println!("{:?}", hi_times_two);
            // hi_values[0] = hi_values[0];
            // hi_values[1] = hi_values[1];
            // hi_values[2] = hi_values[2];
            // hi_values[3] = hi_values[3];

            // hi = aarch64::vld1q_u32(hi_values.as_ptr());

            // perform lower prod and ensure it's in the field
            // let mut lo = aarch64::vmulq_u32(lhs, packed_scalar);
            let lo = aarch64::vaddq_u32(
                aarch64::vandq_u32(aarch64::vmulq_u32(lhs, packed_scalar), packed_modulus),
                aarch64::vshrq_n_u32(aarch64::vmulq_u32(lhs, packed_scalar), 31),
            );
            // let gte_mask = unsafe { aarch64::vcgeq_u32(lo, packed_modulus) };
            // lo = unsafe { aarch64::vsubq_u32(lo, aarch64::vandq_u32(packed_modulus, gte_mask)) };
            // let mut lo_values: [u32; 4] = [
            //     unsafe { aarch64::vgetq_lane_u32(lo, 0) },
            //     unsafe { aarch64::vgetq_lane_u32(lo, 1) },
            //     unsafe { aarch64::vgetq_lane_u32(lo, 2) },
            //     unsafe { aarch64::vgetq_lane_u32(lo, 3) },
            // ];
            // assert_eq!(
            //     lo_values[0],
            //     ((((values[0] as u64) * (scalar as u64)) & 0xFFFFFFFF) % M31_MODULUS as u64) as u32
            // );
            sum_vectors(&mut hi, &lo, &packed_modulus);
            // hi_values = [
            //     unsafe { aarch64::vgetq_lane_u32(hi, 0) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 1) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 2) },
            //     unsafe { aarch64::vgetq_lane_u32(hi, 3) },
            // ];
            // assert_eq!(
            //     hi_values[0],
            //     (((values[0] as u64) * (scalar as u64)) % M31_MODULUS as u64) as u32
            // );

            aarch64::vst1q_u32(values.as_mut_ptr().add(step), hi);
        }
        // unsafe {
        //     // Unrolling the loop to handle 8 elements per iteration (twice the original size).
        //     // First set of 4 elements
        //     let lhs_0 = aarch64::vld1q_u32(values.as_ptr().add(step));
        //     let upper_0 = aarch64::vreinterpretq_u32_s32(aarch64::vqdmulhq_s32(
        //         aarch64::vreinterpretq_s32_u32(lhs_0),
        //         aarch64::vreinterpretq_s32_u32(packed_scalar),
        //     ));
        //     let lower_0 = aarch64::vmulq_u32(lhs_0, packed_scalar);
        //     let t_0 = aarch64::vmlsq_u32(lower_0, upper_0, packed_modulus);
        //     let res_0 = aarch64::vminq_u32(
        //         aarch64::vmlsq_u32(lower_0, upper_0, packed_modulus),
        //         aarch64::vsubq_u32(t_0, packed_modulus),
        //     );
        //     aarch64::vst1q_u32(values.as_mut_ptr().add(step), res_0);

        //     // Second set of 4 elements
        //     let lhs_1 = aarch64::vld1q_u32(values.as_ptr().add(step + 4));
        //     let upper_1 = aarch64::vreinterpretq_u32_s32(aarch64::vqdmulhq_s32(
        //         aarch64::vreinterpretq_s32_u32(lhs_1),
        //         aarch64::vreinterpretq_s32_u32(packed_scalar),
        //     ));
        //     let lower_1 = aarch64::vmulq_u32(lhs_1, packed_scalar);
        //     let t_1 = aarch64::vmlsq_u32(lower_1, upper_1, packed_modulus);
        //     let res_1 = aarch64::vminq_u32(
        //         aarch64::vmlsq_u32(lower_1, upper_1, packed_modulus),
        //         aarch64::vsubq_u32(t_1, packed_modulus),
        //     );
        //     aarch64::vst1q_u32(values.as_mut_ptr().add(step + 4), res_1);
        // }
    }
}
