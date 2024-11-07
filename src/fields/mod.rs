use ark_ff::Field;

pub mod baby_bear;
pub mod m31;

#[cfg(target_arch = "aarch64")]
pub mod aarch64_neon;

pub trait VecOpsField: Field {
    fn reduce_sum(vec: &[Self]) -> Self;
    fn scalar_mult(vec: &mut [Self], scalar: Self);
}
