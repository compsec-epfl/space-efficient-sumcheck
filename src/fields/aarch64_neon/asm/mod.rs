use ark_std::arch::asm;

use crate::fields::m31::reduce_sum_naive;

pub fn reduce_sum_32_bit_modulus_asm(values: &[u32], modulus: u32) -> u32 {
    let modulus: *const u32 = [modulus; 4].as_ptr();
    let mut sums: [u32; 4] = [0; 4];
    for step in (0..values.len()).step_by(4) {
        let vals: *const u32 = unsafe { values.as_ptr().add(step) };

        // TODO (z-tech): Again this should be unrolled, it's also important to understand if these loads / writes are not optimal
        unsafe {
            asm!(
                // Load accumulated sums into register v0
                "ldr q0, [{0}]",

                // Load the new values into register v1
                "ldr q1, [{1}]",

                // Load the modulus into register v3
                "ldr q3, [{2}]",

                // Add values to accumulated sums and put result into v0
                "add v0.4s, v0.4s, v1.4s",

                // Subtract the modulus from the result and put it in v2
                "sub v2.4s, v0.4s, v3.4s",

                // Keep the minimum of those operations
                "umin v0.4s, v0.4s, v2.4s",

                // Load it back into sum accumulator
                "st1 {{v0.4s}}, [{0}]",

                inout(reg) sums.as_mut_ptr() => _,
                in(reg) vals,
                in(reg) modulus,
            );
        }
    }

    let arr: [u32; 4] = unsafe { core::mem::transmute(sums) };
    reduce_sum_naive(&arr)
}

#[cfg(test)]
mod tests {
    use crate::fields::{
        aarch64_neon::reduce_sum_32_bit_modulus_asm,
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
            M31::from(reduce_sum_32_bit_modulus_asm(
                &random_field_values_u32,
                M31_MODULUS
            ))
        );
    }
}
