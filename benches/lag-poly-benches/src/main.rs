use ark_ff::{
    fields::{Fp128, MontBackend, MontConfig},
    Field,
};
use std::env;

use space_efficient_sumcheck::{interpolation::LagrangePolynomial, messages::VerifierMessages};

#[derive(MontConfig)]
#[modulus = "143244528689204659050391023439224324689"] // q = 143244528689204659050391023439224324689
#[generator = "2"]
pub struct FieldConfig128;
pub type Field128 = Fp128<MontBackend<FieldConfig128, 2>>;

fn random_field_elements<F: Field>(n: usize) -> Vec<F> {
    // Create a random number generator
    let mut rng = ark_std::test_rng();

    // Generate n random field elements
    (0..n).map(|_| F::rand(&mut rng)).collect()
}

fn main() {
    let argsv: Vec<String> = env::args().collect();
    let num_vars = argsv[1].parse::<usize>().unwrap();
    let mut lag_poly = LagrangePolynomial::new(VerifierMessages::new(&random_field_elements::<
        Field128,
    >(num_vars)));
    while let Some(_element) = lag_poly.next() {}
}
