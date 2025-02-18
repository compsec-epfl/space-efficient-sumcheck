use ark_std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::fields::{
    m31::{M31, M31_MODULUS},
    vec_ops::VecOps,
};

#[cfg(target_arch = "aarch64")]
use crate::fields::aarch64_neon;

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

impl VecOps for M31 {
    fn reduce_sum(vec: &[M31]) -> Self {
        // #[cfg(target_arch = "aarch64")]
        return M31 {
            value: aarch64_neon::reduce_sum_32_bit_modulus(
                unsafe { from_raw_parts(vec.as_ptr() as *mut u32, vec.len()) },
                M31_MODULUS,
            ),
        };

        #[cfg(not(target_arch = "aarch64"))]
        M31::from(reduce_sum_naive(unsafe {
            from_raw_parts_mut(vec.as_ptr() as *mut u32, vec.len())
        }))
    }

    fn scalar_mult(vec: &mut [Self], scalar: M31) {
        #[cfg(target_arch = "aarch64")]
        aarch64_neon::scalar_mult_32_bit_modulus(
            unsafe { from_raw_parts_mut(vec.as_mut_ptr() as *mut u32, vec.len()) },
            scalar.to_u32(),
            M31_MODULUS,
        );

        #[cfg(not(target_arch = "aarch64"))]
        for elem in vec.iter_mut() {
            *elem = *elem * scalar;
        }
    }
}
