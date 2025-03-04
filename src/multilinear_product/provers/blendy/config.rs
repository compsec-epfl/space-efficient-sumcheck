use ark_ff::Field;

use crate::{prover::ProductProverConfig, streams::Stream};

pub struct BlendyProductProverConfig<F, S>
where
    F: Field,
    S: Stream<F>,
{
    pub num_stages: usize,
    pub num_variables: usize,
    pub claim: F,
    pub stream_p: S,
    pub stream_q: S,
}

impl<'a, F, S> BlendyProductProverConfig<F, S>
where
    F: Field,
    S: Stream<F>,
{
    pub fn new(
        claim: F,
        num_stages: usize,
        num_variables: usize,
        stream_p: S,
        stream_q: S,
    ) -> Self {
        Self {
            claim,
            num_stages,
            num_variables,
            stream_p,
            stream_q,
        }
    }
}

impl<F: Field, S: Stream<F>> ProductProverConfig<F, S> for BlendyProductProverConfig<F, S> {
    fn default(claim: F, num_variables: usize, stream_p: S, stream_q: S) -> Self {
        Self {
            claim,
            num_stages: 2, // DEFAULT
            num_variables,
            stream_p,
            stream_q,
        }
    }
}
