//! The sumcheck protocol is an IP that
//!

pub mod boolean_hypercube;
pub mod polynomial;
pub mod proof;
pub mod prover;

/// The <RUNTIME> (<SPACECOST>) prover implementation.
pub mod basic_prover;
pub mod space_prover;

pub use boolean_hypercube::BooleanHypercube;
pub use polynomial::SumcheckMultivariatePolynomial;
pub use proof::Sumcheck;
pub use prover::Prover;

pub use basic_prover::BasicProver;
pub use space_prover::SpaceProver;

// #[cfg(test)]
// mod tests;
