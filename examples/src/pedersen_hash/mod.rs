// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use winterfell::{
    math::{
        curves::curve_f63,
        fields::f63::BaseElement,
        FieldElement,
    },
    ProofOptions, Prover, StarkProof, VerifierError,
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use log::debug;
#[cfg(feature = "std")]
use std::time::Instant;
#[cfg(feature = "std")]
use winterfell::{math::log2, Trace, TraceTable};

use super::utils::{
    ecc::{self, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH},
    field,
};

mod hash;
pub(crate) use hash::{
    get_constant_points
};

mod air;
use air::{PedersenHashAir, PublicInputs};

mod prover;
use prover::PedersenHashProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, _sequence_length: usize) -> Box<dyn Example> {

    let num_queries = options.num_queries.unwrap_or(8);
    let blowup_factor = options.blowup_factor.unwrap_or(2);
    Box::new(PedersenHashExample::new(
        options.to_proof_options(num_queries, blowup_factor),
    ))
}

pub struct PedersenHashExample {
    options: ProofOptions,
}

impl PedersenHashExample {
    pub fn new(options: ProofOptions) -> PedersenHashExample {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        PedersenHashExample {
            options,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for PedersenHashExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing CairoCpu step up to 16th term\n\
            ---------------------"
        );

        // create a prover
        let prover = PedersenHashProver::new(self.options.clone());

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
        winterfell::verify::<<PedersenHashProver as Prover>::Air>(proof, PublicInputs{})
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<<PedersenHashProver as Prover>::Air>(proof, PublicInputs{})
    }
}
