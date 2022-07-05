// Copyright (c) Facebook, Inc. and its affiliates.
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
use air::{CairoCpuAir, PublicInputs};

mod prover;
use prover::CairoCpuProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 33;

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, sequence_length: usize) -> Box<dyn Example> {
    Box::new(CairoCpuExample::new(
        options.to_proof_options(8, 2),
    ))
}

pub struct CairoCpuExample {
    options: ProofOptions,
}

impl CairoCpuExample {
    pub fn new(options: ProofOptions) -> CairoCpuExample {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        CairoCpuExample {
            options,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for CairoCpuExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing CairoCpu step up to 16th term\n\
            ---------------------"
        );

        // create a prover
        let prover = CairoCpuProver::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace();

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
        winterfell::verify::<CairoCpuAir>(proof, PublicInputs{})
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<CairoCpuAir>(proof, PublicInputs{})
    }
}
