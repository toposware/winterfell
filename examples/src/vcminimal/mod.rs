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

const TRACE_WIDTH: usize = 4;

// VIRTUAL COLUMN MINIMAL EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, a: u128, b: u128,  num_steps: usize) -> Box<dyn Example> {
    assert!(
        num_steps.is_power_of_two(),
        "sequence length must be a power of 2"
    );
    Box::new(VCMinimalExample::new(
        //ALEX: inconsistency, not sure if on purpose
        options.to_proof_options(options.num_queries.unwrap_or(3), options.blowup_factor.unwrap_or(2)),
        [a,b],
        num_steps,
    ))
}

pub struct VCMinimalExample {
    options: ProofOptions,
    inputs: [u128;2],
    num_steps: usize,
}

impl VCMinimalExample {
    pub fn new(options: ProofOptions, inputs: [u128;2], num_steps: usize) -> VCMinimalExample {
        VCMinimalExample {
            options,
            inputs,
            num_steps
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
        let prover = VCMinimalProver::new(self.options.clone(), self.inputs);

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.num_steps);

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
            PublicInputs{input: [BaseElement::from(self.inputs[0]),BaseElement::from(self.inputs[1])]}
        )
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<VCMinimalAir>(
            proof, 
            PublicInputs{input: [BaseElement::from(self.inputs[0]+42),BaseElement::from(self.inputs[1])]}
        )
    }
}
