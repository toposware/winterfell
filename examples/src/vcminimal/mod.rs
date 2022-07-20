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
use air::{VCMinimalAir, PublicInputs};

mod prover;
use prover::VCMinimalProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

// VIRTUAL COLUMN MINIMAL EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, initial: u128, num_steps: usize, width: usize, real_width: usize) -> Box<dyn Example> {
    assert!(
        num_steps.is_power_of_two(),
        "sequence length must be a power of 2"
    );
    Box::new(VCMinimalExample::new(
        options.to_proof_options(options.num_queries.unwrap_or(3), options.blowup_factor.unwrap_or(2)),
        initial,
        num_steps,
        width,
        real_width
    ))
}

pub struct VCMinimalExample {
    options: ProofOptions,
    input: u128,
    num_steps: usize,
    width: usize,
    real_width: usize
}

impl VCMinimalExample {
    pub fn new(options: ProofOptions, input: u128, num_steps: usize, width: usize, real_width: usize) -> VCMinimalExample {
        VCMinimalExample {
            options,
            input,
            num_steps,
            width,
            real_width
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for VCMinimalExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating virtual column minimal example proof up to 8th term\n\
            ---------------------"
        );

        // create a prover
        let prover = VCMinimalProver::new(self.options.clone(), self.input);

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.num_steps, self.width, self.real_width);

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
        winterfell::verify::<VCMinimalAir>(
            proof, 
            PublicInputs{input: BaseElement::from(self.input)}
        )
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<VCMinimalAir>(
            proof, 
            PublicInputs{input: BaseElement::from(self.input + 42)}
        )
    }
}
