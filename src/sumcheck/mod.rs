//! The sumcheck protocol is an IP that
//!

pub mod proof;
pub mod prover;

/// The <RUNTIME> (<SPACECOST>) prover implementation.
pub mod basic_prover;

pub use prover::{Prover, ProverMsgs};
pub use basic_prover::BasicProver;

// #[cfg(test)]
// mod tests;
