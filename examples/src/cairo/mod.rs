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

const TRACE_WIDTH: usize = 218;
const AUX_WIDTH: usize = 33;
const NB_OFFSET_COLUMNS: usize = 4;
const NB_MEMORY_COLUMN_PAIRS: usize = 29;

const OFFSET_COLUMNS: [usize; NB_OFFSET_COLUMNS] = [16, 17, 18, 33];

const SORTED_OFFSET_COLUMNS: [usize; NB_OFFSET_COLUMNS] = [34, 35, 36, 37];

const MEMORY_COLUMNS: [(usize, usize); NB_MEMORY_COLUMN_PAIRS] = [
    (19, 20),
    (21, 22),
    (23, 24),
    (25, 26),
    (38, 39),
    (50, 51),
    (52, 53),
    (54, 55),
    (56, 57),
    (58, 59),
    (60, 61),
    (62, 63),
    (64, 65),
    (66, 67),
    (68, 69),
    (70, 71),
    (72, 73),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 83),
    (84, 85),
    (86, 87),
    (88, 89),
    (90, 91),
    (92, 93),
    (94, 95),
    (96, 97),
];

const SORTED_MEMORY_COLUMNS: [(usize, usize); NB_MEMORY_COLUMN_PAIRS] = [
    (40, 41),
    (42, 43),
    (44, 45),
    (46, 47),
    (48, 49),
    (170, 171),
    (172, 173),
    (174, 175),
    (176, 177),
    (178, 179),
    (180, 181),
    (182, 183),
    (184, 185),
    (186, 187),
    (188, 189),
    (190, 191),
    (192, 193),
    (194, 195),
    (196, 197),
    (198, 199),
    (200, 201),
    (202, 203),
    (204, 205),
    (206, 207),
    (208, 209),
    (210, 211),
    (212, 213),
    (214, 215),
    (216, 217),
];

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

        // read rescue built-in pointer values
        reader.lock().unwrap().read_line(&mut line).unwrap();
        let rescue_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            rescue_pointer_values.len() == 2,
            "Wrong number of rescue pointer values provided."
        );

        // create a prover
        let prover = CairoProver::new(
            self.options.clone(),
            bytecode,
            register_values,
            rescue_pointer_values,
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

        reader.lock().unwrap().read_line(&mut line).unwrap();
        let rescue_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            rescue_pointer_values.len() == 2,
            "Wrong number of rescue pointer values provided."
        );

        // println!("{:#?}", bytecode);
        let pub_inputs = PublicInputs {
            bytecode: bytecode,
            register_values: register_values,
            rescue_pointer_values: rescue_pointer_values,
        };
        winterfell::verify::<CairoAir>(proof, pub_inputs)
    }

    //TODO: implement wrong trace checking
    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        Err(VerifierError::InconsistentBaseField)
    }
}
