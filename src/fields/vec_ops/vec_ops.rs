use ark_ff::Field;

pub trait VecOps: Field {
    fn reduce_sum(vec: &[Self]) -> Self;
    fn scalar_mult(vec: &mut [Self], scalar: Self);
}
