use ark_ff::fields::{Fp64, MontBackend, MontConfig};

#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]

pub struct F19Config;
pub type F19 = Fp64<MontBackend<F19Config, 1>>;
