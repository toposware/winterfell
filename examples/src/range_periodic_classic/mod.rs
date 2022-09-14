// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, log2, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::{PublicInputs, RangeAir};

mod prover;
use prover::RangeProver;

#[cfg(test)]
mod tests;

// RANGE WITH DIVISORS EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, sequence_length: usize) -> Box<dyn Example> {
    Box::new(RangeExample::new(
        sequence_length,
        options.to_proof_options(28, 4),
    ))
}

pub struct RangeExample {
    options: ProofOptions,
    sequence_length: usize,
}

impl RangeExample {
    pub fn new(sequence_length: usize, options: ProofOptions) -> RangeExample {
        RangeExample {
            options,
            sequence_length,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for RangeExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing range check sequence of length {}\n\
            ---------------------",
            self.sequence_length,
        );

        // create a prover
        let prover = RangeProver::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.sequence_length);

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
            input_value: BaseElement::new(0),
        };
        winterfell::verify::<RangeAir>(proof, pub_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            input_value: BaseElement::ONE,
        };
        winterfell::verify::<RangeAir>(proof, pub_inputs)
    }
}
