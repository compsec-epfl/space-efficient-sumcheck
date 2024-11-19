use ark_std::{
    arch::{aarch64, asm},
    cmp,
    mem::transmute,
};

use crate::fields::m31::M31_MODULUS;

// #[inline(always)]
// unsafe fn reduce_sum_32_bit_asm(vec: &[u32], modulus: u32) -> u32 {
//     println!("regardes: {:?}, {:?}", vec.as_ptr(), vec.as_ptr().add(vec.len()));
//     let mut sums: [u32; 4] = [0, 0, 0, 0];
//     asm!(
//         // Load pointers to start and end
//         "mov x9, {0}",
//         "mov x10, {1}",

//         // Initialize sum to zero
//         "eor v5.16b, v5.16b, v5.16b",

//         // Load the modulus into q4
//         "dup v4.4s, {2:w}",

//         // Loop start
//         "1:",

//         // Compare start ptr with end pointer, if equal jump to return
//         "cmp x9, x10",
//         "beq 2f",

//             // Load 16 u32 (4 each) into v0, v1, v2, v3
//             "ld1 {{ v0.4s, v1.4s, v2.4s, v3.4s }}, [{0}], #64",
//             // "add x9, x9, #4",
//             "mov x9, {0}",

//             // // Accumulate v0 and v1
//             // "add v0.4s, v0.4s, v1.4s",
//             // "sub v1.4s, v0.4s, v4.4s",
//             // "umin v0.4s, v0.4s, v1.4s",

//             // // Accumulate v2 and v3
//             // "add v2.4s, v2.4s, v3.4s",
//             // "sub v3.4s, v2.4s, v4.4s",
//             // "umin v2.4s, v2.4s, v3.4s",

//             // // Accumulate sum and v0
//             // "add v0.4s, v0.4s, v5.4s",
//             // "sub v1.4s, v0.4s, v4.4s",
//             // "umin v0.4s, v0.4s, v1.4s",

//             // // Accumulate sum and v2
//             // "add v2.4s, v2.4s, v5.4s",
//             // "sub v3.4s, v2.4s, v4.4s",
//             // "umin v2.4s, v2.4s, v3.4s",

//             // // Accumulate v0 and v2
//             // "add v0.4s, v0.4s, v2.4s",
//             // "sub v1.4s, v0.4s, v4.4s",
//             // "umin v0.4s, v0.4s, v2.4s",

//         // Branch back to loop start
//         "b 1b",

//         // Loop end label
//         "2:",

//         // Write the reduced sum back into result
//         "st1 {{v0.4s}}, [{3}]",

//         in(reg) vec.as_ptr(),
//         in(reg) vec.as_ptr().add(vec.len()),
//         in(reg) modulus,
//         inout(reg) sums.as_mut_ptr() => _,
//         options(nostack),
//     );

//     assert_eq!(sum[0], 1);

//     // sum and reduce across lanes
//     let mut sum = sums[0] + sums[1];
//     if sum >= modulus {
//         sum = sum - modulus;
//     }
//     sum = sum + sums[2];
//     if sum >= modulus {
//         sum = sum - modulus;
//     }
//     sum = sum + sums[3];
//     if sum >= modulus {
//         sum = sum - modulus;
//     }

//     // accumulated into one u32
//     sum
// }

// #[inline(always)]
// unsafe fn sum_8_vectors_asm(v0: *const u32, modulus: *const u32) -> [u32; 4] {
//     let mut dest: [u32; 4] = [0, 0, 0, 0];
//     asm!(
//         // Load 16 u32 (4 each) into v0, v1, v2, v3
//         "ld1 {{ v0.4s, v1.4s, v2.4s, v3.4s }}, [{0}]",

//         // Load the modulus into q4
//         "ldr q4, [{1}]",

//         "add v0.4s, v0.4s, v1.4s",
//         "add v2.4s, v2.4s, v3.4s",

//         "sub v1.4s, v0.4s, v4.4s",
//         "sub v3.4s, v2.4s, v4.4s",
//         "umin v0.4s, v0.4s, v1.4s",
//         "umin v2.4s, v2.4s, v3.4s",

//         // repeat 2
//         "add v0.4s, v0.4s, v2.4s",
//         "sub v1.4s, v0.4s, v4.4s",
//         "umin v0.4s, v0.4s, v1.4s",

//         // repeat 1
//         // "add v0.4s, v0.4s, v4.4s",
//         // "sub v1.4s, v0.4s, v8.4s",
//         // "umin v0.4s, v0.4s, v1.4s",

//         "st1 {{v0.4s}}, [{2}]",

//         in(reg) v0,
//         // in(reg) v0.add(16),
//         in(reg) modulus,
//         inout(reg) dest.as_mut_ptr() => _,
//         options(nostack),
//     );
//     return dest;
// }

#[inline(always)]
unsafe fn sum_8_vectors_asm(
    v0: *const u32,
    modulus: *const u32,
    acc0: &mut [u32; 4],
    acc1: &mut [u32; 4],
    acc2: &mut [u32; 4],
    acc3: &mut [u32; 4],
) {
    asm!(
        // Load vectors v0, v1, v2, v3 into registers q0, q1, q2, q3
        // Load vectors v4, v5, v6, v7 into registers q4, q5, q6, q7
        "ld1 {{ v0.4s, v1.4s, v2.4s, v3.4s }}, [{0}]",
        "ld1 {{ v4.4s, v5.4s, v6.4s, v7.4s }}, [{1}]",

        // Load the modulus into q4
        "ldr q8, [{2}]",

        // Addition
        // Add v0 and v1: v0 = v0 + v1
        // Add v2 and v3: v2 = v2 + v3
        // Add v4 and v5: v4 = v4 + v5
        // Add v6 and v7: v6 = v6 + v7
        "add v0.4s, v0.4s, v1.4s",
        "add v2.4s, v2.4s, v3.4s",
        "add v4.4s, v4.4s, v5.4s",
        "add v6.4s, v6.4s, v7.4s",

        // Mod into field
        // Sub v0 and v8: v1 = v0 - v4
        // Sub v2 and v8: v3 = v2 - v4
        // Sub v4 and v8: v1 = v0 - v4
        // Sub v6 and v8: v3 = v2 - v4
        "sub v1.4s, v0.4s, v8.4s",
        "sub v3.4s, v2.4s, v8.4s",
        "sub v5.4s, v4.4s, v8.4s",
        "sub v7.4s, v6.4s, v8.4s",
        "umin v0.4s, v0.4s, v1.4s",
        "umin v2.4s, v2.4s, v3.4s",
        "umin v4.4s, v4.4s, v5.4s",
        "umin v6.4s, v6.4s, v7.4s",

        // v1, v3, v5, v7 are now free
        "ldr q1, [{3}]",
        "ldr q3, [{4}]",
        "ldr q5, [{5}]",
        "ldr q7, [{6}]",

        "add v0.4s, v0.4s, v1.4s",
        "add v2.4s, v2.4s, v3.4s",
        "add v4.4s, v4.4s, v5.4s",
        "add v6.4s, v6.4s, v7.4s",

        "sub v1.4s, v0.4s, v8.4s",
        "sub v3.4s, v2.4s, v8.4s",
        "sub v5.4s, v4.4s, v8.4s",
        "sub v7.4s, v6.4s, v8.4s",
        "umin v0.4s, v0.4s, v1.4s",
        "umin v2.4s, v2.4s, v3.4s",
        "umin v4.4s, v4.4s, v5.4s",
        "umin v6.4s, v6.4s, v7.4s",

        // // repeat 2
        // "add v0.4s, v0.4s, v2.4s",
        // "sub v1.4s, v0.4s, v8.4s",
        // "umin v0.4s, v0.4s, v1.4s",

        // "add v4.4s, v4.4s, v6.4s",
        // "sub v5.4s, v4.4s, v8.4s",
        // "umin v4.4s, v4.4s, v5.4s",

        // // repeat 1
        // "add v0.4s, v0.4s, v4.4s",
        // "sub v1.4s, v0.4s, v8.4s",
        // "umin v0.4s, v0.4s, v1.4s",

        "st1 {{v0.4s}}, [{3}]",
        "st1 {{v2.4s}}, [{4}]",
        "st1 {{v4.4s}}, [{5}]",
        "st1 {{v6.4s}}, [{6}]",

        in(reg) v0,
        in(reg) v0.add(16),
        in(reg) modulus,
        inout(reg) acc0.as_mut_ptr() => _,
        inout(reg) acc1.as_mut_ptr() => _,
        inout(reg) acc2.as_mut_ptr() => _,
        inout(reg) acc3.as_mut_ptr() => _,
        options(nostack),
    );
}

pub fn reduce_sum_naive(vec: &[u32]) -> u32 {
    let sum: u32 = vec.iter().fold(0, |acc, &x| {
        let tmp = acc + x;
        if tmp < M31_MODULUS {
            return tmp;
        } else {
            return tmp - M31_MODULUS;
        }
    });
    sum
}

#[inline(always)]
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

#[inline(always)]
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
    let p = [modulus; 4];
    // let packed_modulus: aarch64::uint32x4_t =
    //     unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([modulus; 4]) };
    // let mut sums: aarch64::uint32x4_t = unsafe { aarch64::vdupq_n_u32(0) };

    let mut acc0: [u32; 4] = [0, 0, 0, 0];
    let mut acc1: [u32; 4] = [0, 0, 0, 0];
    let mut acc2: [u32; 4] = [0, 0, 0, 0];
    let mut acc3: [u32; 4] = [0, 0, 0, 0];

    // Note: it's a big ugly function bc it must be unrolled to fill up available registers
    for step in (0..values.len()).step_by(64) {
        // sum the first 8 vectors into v0, v2, v4, v6
        // let mut v0 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step)) };
        // let v1 = unsafe { aarch64::vld1q_u32(values.as_ptr().add(step + 4)) };
        // let mut dest: [u32; 4] = [0, 0, 0, 0];

        unsafe {
            sum_8_vectors_asm(
                values.as_ptr().add(step),
                p.as_ptr(),
                &mut acc0,
                &mut acc1,
                &mut acc2,
                &mut acc3,
            )
        }
        // acc0 = result.0;
        // acc1 = result.1;
        // acc2 = result.2;
        // acc3 = result.3;
        // let mut v0 = unsafe {
        //     aarch64::vld1q_u32(sum_8_vectors_asm(values.as_ptr().add(step), p.as_ptr()).as_ptr())
        // };

        // sum the accumulated sums into sums
        // sum_vectors(&mut sums, &v0, &packed_modulus);
        // sum_vectors(&mut sums, &v8, &packed_modulus);
    }

    reduce_sum_naive(&[
        acc0[0], acc0[1], acc0[2], acc0[3], acc1[0], acc1[1], acc1[2], acc1[3], acc2[0], acc2[1],
        acc2[2], acc2[3], acc3[0], acc3[1], acc3[2], acc3[3],
    ])
    // let r = sum_lanes(&sums);
    // let test = unsafe { reduce_sum_32_bit_asm(values, modulus) };
    // assert_eq!(test, r);
    // r
}

pub fn scalar_mult_32_bit_modulus(values: &mut [u32], scalar: u32, modulus: u32) {
    let packed_modulus: aarch64::uint32x4_t =
        unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([modulus; 4]) };
    let packed_scalar: aarch64::uint32x4_t =
        unsafe { transmute::<[u32; 4], aarch64::uint32x4_t>([scalar; 4]) };
    for step in (0..values.len()).step_by(4) {
        unsafe {
            let lhs = aarch64::vld1q_u32(values.as_ptr().add(step));
            let upper = aarch64::vreinterpretq_u32_s32(aarch64::vqdmulhq_s32(
                aarch64::vreinterpretq_s32_u32(lhs),
                aarch64::vreinterpretq_s32_u32(packed_scalar),
            ));
            let lower = aarch64::vmulq_u32(lhs, packed_scalar);
            let t = aarch64::vmlsq_u32(lower, upper, packed_modulus);
            let res = aarch64::vminq_u32(
                aarch64::vmlsq_u32(lower, upper, packed_modulus),
                aarch64::vsubq_u32(t, packed_modulus),
            );
            aarch64::vst1q_u32(values.as_mut_ptr().add(step), res);
        }
    }
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
