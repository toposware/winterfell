// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use rand_utils::rand_array;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, log2, ExtensionOf, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::CairoAir;

mod prover;
use prover::CairoProver;

#[cfg(test)]
mod tests;

mod custom_trace_table;
pub use custom_trace_table::RapTraceTable;

use crate::utils::print_trace;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 50;
const AUX_WIDTH: usize = 9;

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, trace_file_path: String) -> Box<dyn Example> {
    Box::new(CairoExample::new(
        options.to_proof_options(28, 8),
        trace_file_path,
    ))
}

pub struct CairoExample {
    options: ProofOptions,
    trace_file_path: String,
}

impl CairoExample {
    pub fn new(options: ProofOptions, trace_file_path: String) -> CairoExample {
        CairoExample {
            options,
            trace_file_path,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for CairoExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating a proof for running a Cairo program\n\
            ---------------------"
        );

        // create a prover
        let prover = CairoProver::new(self.options.clone());

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace_from_file(&self.trace_file_path);

        let trace_width = trace.width();
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace_width,
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // TODO: make it possible to print the custom trace
        // print_trace(&trace, 1, 0, 0..trace.width());

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<CairoAir>(proof, ())
    }

    //TODO: implement wrong trace checking
    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        Err(VerifierError::InconsistentBaseField)
    }
}
