use ark_ff::Field;

pub trait EvaluationStream<F: Field> {
    fn get_claimed_sum(&self) -> F;
    fn get_evaluation(&self, point: usize) -> F;
    fn get_num_variables(&self) -> usize;
}
