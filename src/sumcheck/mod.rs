//! The sumcheck protocol is an IP that
//!

pub mod boolean_hypercube;
pub mod proof;
pub mod prover;
pub mod sumcheck_polynomial;

/// The <RUNTIME> (<SPACECOST>) prover implementation.
pub mod basic_prover;

pub use basic_prover::BasicProver;
pub use boolean_hypercube::BooleanHypercube;
pub use prover::Prover;
pub use sumcheck_polynomial::SumcheckPolynomial;

// #[cfg(test)]
// mod tests;
