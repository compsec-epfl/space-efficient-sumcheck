mod asm;
mod intrinsics;

pub use asm::reduce_sum_32_bit_modulus_asm;
pub use intrinsics::{reduce_sum_32_bit_modulus, scalar_mult_32_bit_modulus};
