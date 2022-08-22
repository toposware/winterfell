// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f64::BaseElement, log2, ExtensionOf, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::{CairoAir, PublicInputs};

mod prover;
use prover::CairoProver;

#[cfg(test)]
mod tests;

mod custom_trace_table;
pub use custom_trace_table::RapTraceTable;

use crate::utils::print_trace;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Mutex;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 54;
const AUX_WIDTH: usize = 10;

// CAIRO EXAMPLE
// ================================================================================================

pub fn get_example(
    options: ExampleOptions,
    trace_file_path: String,
    public_input_file_path: String,
) -> Box<dyn Example> {
    Box::new(CairoExample::new(
        options.to_proof_options(28, 8),
        trace_file_path,
        public_input_file_path,
    ))
}

pub struct CairoExample {
    options: ProofOptions,
    trace_file_path: String,
    public_input_file_path: String,
}

impl CairoExample {
    pub fn new(
        options: ProofOptions,
        trace_file_path: String,
        public_input_file_path: String,
    ) -> CairoExample {
        CairoExample {
            options,
            trace_file_path,
            public_input_file_path,
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

        // read bytecode from file
        let file = File::open(&self.public_input_file_path).expect("Cannot open the file.");
        let reader = Mutex::new(BufReader::new(file));
        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode_length = usize::from_str(&line).unwrap();
        line.clear();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            2 * bytecode_length == bytecode.len(),
            "Wrong number of values provided."
        );
        line.clear();

        // read register boundary values
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let register_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            register_values.len() == 4,
            "Wrong number of register boundary values provided."
        );
        line.clear();

        // read built-in pointers values
        reader.lock().unwrap().read_line(&mut line).unwrap();
        let output_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            output_pointer_values.len() == 2,
            "Wrong number of output pointer values provided."
        );

        // create a prover
        let prover = CairoProver::new(
            self.options.clone(),
            bytecode,
            register_values,
            output_pointer_values,
        );

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
        // read bytecode from file
        let file = File::open(&self.public_input_file_path).expect("Cannot open the file.");
        let reader = Mutex::new(BufReader::new(file));
        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode_length = usize::from_str(&line).unwrap();
        line.clear();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            2 * bytecode_length == bytecode.len(),
            "Wrong number of values provided."
        );
        line.clear();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let register_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            register_values.len() == 4,
            "Wrong number of register boundary values provided."
        );
        line.clear();

        // println!("{:#?}", bytecode);
        reader.lock().unwrap().read_line(&mut line).unwrap();
        let output_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            output_pointer_values.len() == 2,
            "Wrong number of output pointer values provided."
        );

        let pub_inputs = PublicInputs {
            bytecode: bytecode,
            register_values: register_values,
            output_pointer_values: output_pointer_values,
        };
        winterfell::verify::<CairoAir>(proof, pub_inputs)
    }

    //TODO: implement wrong trace checking
    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        Err(VerifierError::InconsistentBaseField)
    }
}
