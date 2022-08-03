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
use air::DivisorsExemptionsAir;

mod prover;
use prover::DivisorsExemptionsProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 3;

// FIBONACCI EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, sequence_length: [u64; 2]) -> Box<dyn Example> {
    Box::new(DivisorsExemptionsExample::new(
        sequence_length,
        options.to_proof_options(28, 8),
    ))
}

pub struct DivisorsExemptionsExample {
    options: ProofOptions,
    sequence_length: [u64; 2],
    result: air::PublicInputs,
}

impl DivisorsExemptionsExample {
    pub fn new(sequence_length: [u64; 2], options: ProofOptions) -> DivisorsExemptionsExample {
        assert!(
            sequence_length[0].is_power_of_two(),
            "Fibonacci sequence length must be a power of 2"
        );
        assert!(
            sequence_length[1] <= sequence_length[0]/2,
            "Exponentiation sequence length must be less or equal to half of Fibonacci sequence length"
        );

        // compute Fibonacci sequence
        let now = Instant::now();
        let result_fib = compute_fib_term(sequence_length[0]);
        debug!(
            "Computed Fibonacci sequence up to {}th term in {} ms",
            sequence_length[0],
            now.elapsed().as_millis()
        );

        let now = Instant::now();
        let result_exp = BaseElement::new(2u128).exp(sequence_length[1] as u128 - 1);
        debug!(
            "Computed 2 to the power of {} in {} ms",
            sequence_length[1] - 1,
            now.elapsed().as_millis()
        );

        let result = air::PublicInputs {
            results: [result_fib, result_exp],
            last_exp_step: sequence_length[1],
        };

        DivisorsExemptionsExample {
            options,
            sequence_length,
            result,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for DivisorsExemptionsExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating proof for computing Fibonacci sequence (2 terms per step) up to {}th term and exponentiating 2 up to the 
            power of {}.\n\
            ---------------------",
            self.sequence_length[0],
            self.sequence_length[1] - 1
        );

        // create a prover
        let prover = DivisorsExemptionsProver::new(self.options.clone(), self.sequence_length[1]);

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
        winterfell::verify::<DivisorsExemptionsAir>(proof, self.result.clone())
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        winterfell::verify::<DivisorsExemptionsAir>(
            proof,
            air::PublicInputs {
                results: [
                    self.result.results[0] + BaseElement::ONE,
                    self.result.results[1] + BaseElement::ONE,
                ],
                last_exp_step: self.result.last_exp_step,
            },
        )
    }
}

pub fn compute_fib_term(n: u64) -> BaseElement {
    let mut t0 = BaseElement::ONE;
    let mut t1 = BaseElement::ONE;

    for _ in 0..(n - 1) {
        t1 = t0 + t1;
        core::mem::swap(&mut t0, &mut t1);
    }

    t1
}
