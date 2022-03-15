// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::utils::compute_fib_term;
use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, log2, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::{compress_tuple, FibRapAir, PublicInputs, TRACE_LENGTH, TRACE_WIDTH};

mod prover;
use prover::FibRapProver;

#[cfg(test)]
mod tests;

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions) -> Box<dyn Example> {
    Box::new(FibRapExample::new(options.to_proof_options(28, 8)))
}

pub struct FibRapExample {
    options: ProofOptions,
    sequence_length: usize,
    result: BaseElement,
    rap_challenges: [BaseElement; 2],
}

impl FibRapExample {
    pub fn new(options: ProofOptions) -> FibRapExample {
        let sequence_length = TRACE_LENGTH * 2;
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // compute Fibonacci sequence
        let now = Instant::now();
        // Taking into account tha row sequence_legth/4 is copied to sequence_length/2
        let result = compute_fib_term(sequence_length * 3 / 4);
        debug!(
            "Computed Fibonacci sequence up to {}th term in {} ms",
            sequence_length,
            now.elapsed().as_millis()
        );

        // https://xkcd.com/221/
        let rap_challenges = [BaseElement::new(1), BaseElement::new(1)];

        FibRapExample {
            options,
            sequence_length,
            result,
            rap_challenges,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for FibRapExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing Fibonacci sequence (2 terms per step) up to {}th term\n\
            ---------------------",
            self.sequence_length
        );

        // create a prover
        let prover = FibRapProver::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let mut trace = prover.build_trace(self.sequence_length, self.rap_challenges);

        let trace_width = trace.width();
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace_width,
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(&mut trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let public_inputs = PublicInputs {
            result: self.result,
            rap_challenges: self.rap_challenges,
        };
        winterfell::verify::<FibRapAir>(proof, public_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let public_inputs = PublicInputs {
            result: self.result + BaseElement::ONE,
            rap_challenges: self.rap_challenges,
        };
        winterfell::verify::<FibRapAir>(proof, public_inputs)
    }
}
