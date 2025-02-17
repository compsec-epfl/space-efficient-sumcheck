use ark_ff::Field;

mod memory_mapped;

pub trait EvaluationStream<F: Field> {
    fn claim(&self) -> F;
    fn evaluation(&self, point: usize) -> F;
    fn num_variables(&self) -> usize;
}
