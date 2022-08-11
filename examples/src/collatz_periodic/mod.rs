// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, log2, FieldElement, StarkField},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::{CollatzAir, PublicInputs};

mod prover;
use prover::CollatzProver;

#[cfg(test)]
mod tests;

// Helper function
// Outputs the n-th term of the Collatz sequence of a given integer.
pub(crate) fn compute_collatz_sequence(input_value: usize, sequence_length: usize) -> usize {
    assert!(sequence_length > 0, "Sequence length must be nonzero.");
    assert!(input_value > 0, "Input value must be nonzero.");
    let mut n = input_value;

    for _ in 1..sequence_length {
        if n % 2 == 0 {
            n /= 2;
        } else {
            n = 3 * n + 1;
        }
    }

    n
}

// COLLATZ EXAMPLE
// ================================================================================================

pub fn get_example(
    options: ExampleOptions,
    input_value: usize,
    sequence_length: usize,
) -> Box<dyn Example> {
    Box::new(CollatzExample::new(
        input_value,
        sequence_length,
        options.to_proof_options(28, 4),
    ))
}

pub struct CollatzExample {
    options: ProofOptions,
    input_value: usize,
    final_value: BaseElement,
    sequence_length: usize,
}

impl CollatzExample {
    pub fn new(
        input_value: usize,
        sequence_length: usize,
        options: ProofOptions,
    ) -> CollatzExample {
        // compute Collatz sequence
        let now = Instant::now();
        let final_value = compute_collatz_sequence(input_value, sequence_length);
        debug!(
            "Computed {} terms of the Collatz sequence of {} (result = {}) in {} ms",
            sequence_length,
            input_value,
            final_value,
            now.elapsed().as_millis()
        );

        CollatzExample {
            options,
            input_value,
            final_value: BaseElement::from(final_value as u64),
            sequence_length,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for CollatzExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing {}-th term of Collatz sequence of {}\n\
            ---------------------",
            self.sequence_length, self.input_value,
        );

        // create a prover
        let prover = CollatzProver::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.input_value, self.sequence_length);

        let trace_width = trace.width();
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace_width,
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            input_value: BaseElement::from(self.input_value as u64),
            final_value: self.final_value,
            sequence_length: self.sequence_length,
        };
        winterfell::verify::<CollatzAir>(proof, pub_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            input_value: BaseElement::from(self.input_value as u64),
            final_value: self.final_value + BaseElement::ONE,
            sequence_length: self.sequence_length,
        };
        winterfell::verify::<CollatzAir>(proof, pub_inputs)
    }
}
