pub mod streaming_sumcheck;
pub mod linear_space_sumcheck;
pub mod blended_sumcheck;

use ark_ff::Field;

pub trait SumcheckProver<F: Field> {
    type SumcheckParameters;
}
