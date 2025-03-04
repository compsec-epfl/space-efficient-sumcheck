use ark_ff::Field;

pub trait Stream<F: Field> {
    fn claim(&self) -> F;
    fn evaluation(&self, point: usize) -> F;
    fn num_variables(&self) -> usize;
}
