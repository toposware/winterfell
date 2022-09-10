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
use winterfell::{Air, math::log2, Trace, TraceTable};

use super::utils::{
    ecc::{self, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH},
    field,
};

mod hash;
pub(crate) use hash::{
    get_constant_points
};

mod air;
use air::{MuxAir, ZeroAir, MuxPublicInputs};

mod subsetsumair;
use subsetsumair::{SubsetSumAir, PublicInputs};

mod prover;
use prover::{SubsetSumProver, MuxProver, NUM_CONSTANTS, MUX_LAST_ROW_INDEX};

#[cfg(test)]
mod tests;

type Type<T> = T;

// CONSTANTS
// ================================================================================================

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, _sequence_length: usize) -> Box<dyn Example> {

    let num_queries = options.num_queries.unwrap_or(8);
    let blowup_factor = options.blowup_factor.unwrap_or(2);
    Box::new(SubsetSumExample::new(
        options.to_proof_options(num_queries, blowup_factor),
    ))
}

pub struct SubsetSumExample {
    options: ProofOptions,
}

impl SubsetSumExample {
    pub fn new(options: ProofOptions) -> SubsetSumExample {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        SubsetSumExample {
            options,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for SubsetSumExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing CairoCpu step up to 16th term\n\
            ---------------------"
        );

        // create a prover
        let prover = SubsetSumProver::new(self.options.clone());

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
        winterfell::verify::<<SubsetSumProver as Prover>::Air>(proof, PublicInputs{})
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<<SubsetSumProver as Prover>::Air>(proof, PublicInputs{})
    }
}

// MUX
pub struct MuxExample {
    options: ProofOptions,
}

impl MuxExample {
    pub fn new(options: ProofOptions) -> MuxExample {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        MuxExample {
            options,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for MuxExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing CairoCpu step up to 16th term\n\
            ---------------------"
        );

        // create a prover
        let prover = MuxProver::new(self.options.clone());

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
        winterfell::verify::<<MuxProver as Prover>::Air>(proof, MuxPublicInputs{
            input_left:Type::<<SubsetSumAir<NUM_CONSTANTS> as Air>::PublicInputs>{},
            input_right: Type::<<ZeroAir<MUX_LAST_ROW_INDEX> as Air>::PublicInputs>{}
        })
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<<MuxProver as Prover>::Air>(proof, MuxPublicInputs{
            input_left:Type::<<SubsetSumAir<NUM_CONSTANTS> as Air>::PublicInputs>{},
            input_right: Type::<<ZeroAir<MUX_LAST_ROW_INDEX> as Air>::PublicInputs>{}
        })
    }
}


