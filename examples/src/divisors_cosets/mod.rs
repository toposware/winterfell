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
use air::DivisorsCosetsAir;

mod prover;
use prover::DivisorsCosetsProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 2;

// DIVISORS FOR COSETS EXAMPLE
// ================================================================================================

// The example works as follows: it checks an exponentiation computation, i.e. 2^i for some i. In parallel, there
// is a second raw that is supposed to contain bit values in some places. We do not care where they come from (currently sampled randomly).
// The range_length value says how many range checks are there performed and the offset on what window.
// Example: sequence_length 32, range_length 32 and offset 0 (resp. 1) will check that values in the second column in even (resp odd)
// raws are bits
pub fn get_example(
    options: ExampleOptions,
    sequence_length: u64,
    range_length: u64,
    offset: u64,
) -> Box<dyn Example> {
    Box::new(DivisorsCosetsExample::new(
        sequence_length,
        range_length,
        offset,
        options.to_proof_options(28, 8),
    ))
}

pub struct DivisorsCosetsExample {
    options: ProofOptions,
    sequence_length: u64,
    range_length: u64,
    offset: u64,
    public_inputs: air::PublicInputs,
}

impl DivisorsCosetsExample {
    pub fn new(
        sequence_length: u64,
        range_length: u64,
        offset: u64,
        options: ProofOptions,
    ) -> DivisorsCosetsExample {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );
        assert!(
            range_length.is_power_of_two(),
            "range length must be a power of 2"
        );
        assert!(
            range_length <= sequence_length,
            "range length must be less or equal to sequence length"
        );
        assert!(
            offset < sequence_length / range_length,
            "range check offset must be in the range 0 to total length / subgroup size - 1"
        );

        let now = Instant::now();
        let result_exp = BaseElement::new(2u128).exp(sequence_length as u128 - 1);
        debug!(
            "Computed 2 to the power of {} in {} ms",
            sequence_length - 1,
            now.elapsed().as_millis()
        );

        let public_inputs = air::PublicInputs {
            result: result_exp,
            range_length,
            offset,
        };

        DivisorsCosetsExample {
            options,
            sequence_length,
            range_length,
            offset,
            public_inputs,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for DivisorsCosetsExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing powers of two sequence up to {}th term and range checking in parallel at subtrace
            of size {} and offset {}.\n\
            ---------------------",
            self.sequence_length-1,
            self.range_length,
            self.offset,
        );

        // create a prover
        let prover =
            DivisorsCosetsProver::new(self.options.clone(), self.range_length, self.offset);

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.sequence_length, self.range_length, self.offset);

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
        winterfell::verify::<DivisorsCosetsAir>(proof, self.public_inputs.clone())
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<DivisorsCosetsAir>(
            proof,
            air::PublicInputs {
                result: self.public_inputs.result + BaseElement::ONE,
                range_length: self.public_inputs.range_length,
                offset: self.public_inputs.offset,
            },
        )
    }
}
