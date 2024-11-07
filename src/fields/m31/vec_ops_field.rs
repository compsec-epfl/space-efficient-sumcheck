use ark_std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::fields::{
    m31::{M31, M31_MODULUS},
    VecOpsField,
};

#[cfg(target_arch = "aarch64")]
use crate::fields::aarch64_neon;

impl VecOpsField for M31 {
    fn reduce_sum(vec: &[M31]) -> Self {
        #[cfg(target_arch = "aarch64")]
        return M31 {
            value: aarch64_neon::reduce_sum_32_bit_modulus(
                unsafe { from_raw_parts(vec.as_ptr() as *mut u32, vec.len()) },
                M31_MODULUS,
            ),
        };

        #[cfg(not(target_arch = "aarch64"))]
        {
            let reduced_sum: u32 = vec.iter().fold(0, |acc, &x| {
                let sum = acc + x.to_u32();
                if sum < M31_MODULUS {
                    return sum;
                } else {
                    return sum - M31_MODULUS;
                }
            });
            Self { value: reduced_sum }
        }
    }

    fn scalar_mult(vec: &mut [Self], scalar: M31) {
        #[cfg(target_arch = "aarch64")]
        aarch64_neon::scalar_mult_32_bit_modulus(
            unsafe { from_raw_parts_mut(vec.as_mut_ptr() as *mut u32, vec.len()) },
            scalar.to_u32(),
            M31_MODULUS,
        );

        #[cfg(not(target_arch = "aarch64"))]
        for elem in values.iter_mut() {
            *elem = *elem * scalar;
        }
    }
}
