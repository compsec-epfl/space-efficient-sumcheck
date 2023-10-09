use ark_ff::Field;
use ark_std::borrow::Borrow;
use ark_std::boxed::Box;
use ark_std::vec::Vec;

use crate::sumcheck::prover::{ProverMsgs, RoundMsg};
use crate::sumcheck::Prover;

#[derive(Debug)]
pub struct Sumcheck<F: Field> {
    /// The the messages sent by the prover throughout the protocol
    pub g: Vec<F>,
    /// The the messages sent by the prover throughout the protocol
    pub messages: Vec<RoundMsg<F>>,
    /// The challenges sent by the verifier throughout the protocol
    pub challenges: Vec<F>,
    /// The number of rounds in the protocol.
    rounds: usize,
}

impl<F: Field> Sumcheck<F> {
    pub fn prove<P: Prover<F>>(mut prover: P) -> Self {
        let rounds = prover.rounds();
        let mut messages = Vec::with_capacity(rounds);
        let mut challenges = Vec::with_capacity(rounds);

        let mut verifier_message = None;
        while let Some(message) = prover.next_message(verifier_message) {
            // add the message sent to the transcript
            transcript.append_serializable(b"evaluations", &message);
            // compute the challenge for the next round
            let challenge = transcript.get_challenge(b"challenge");
            verifier_message = Some(challenge);

            // add the message to the final proof
            messages.push(message);
            challenges.push(challenge);
        }

        let rounds = prover.rounds();
        let final_statements = vec![prover.final_statements().unwrap()];
        // Add the final statments to the transcript
        transcript.append_serializable(b"final-statment", &final_statements[0][0]);
        transcript.append_serializable(b"final-statement", &final_statements[0][1]);

        Sumcheck {
            messages,
            challenges,
            rounds,
            final_statements,
        }
    }
}
