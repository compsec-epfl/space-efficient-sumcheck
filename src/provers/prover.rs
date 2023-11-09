use ark_ff::Field;

pub trait Prover<F: Field> {
    fn claimed_sum(&self) -> F;
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)>;
    fn total_rounds(&self) -> usize;
}
