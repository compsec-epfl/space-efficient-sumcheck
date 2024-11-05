use ark_ff::Field;

pub mod baby_bear;
pub mod m31;

pub trait VecOpsField: Field {
    fn reduce_sum(vec: &[Self]) -> Self;
    fn scalar_mult(vec: &mut [Self], scalar: Self);
}
