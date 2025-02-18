mod m31;
mod vec_ops;

#[cfg(target_arch = "aarch64")]
pub mod aarch64_neon;

pub use m31::{reduce_sum_naive, M31, M31_MODULUS};
pub use vec_ops::VecOps;
