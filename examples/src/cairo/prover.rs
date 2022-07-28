// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, CairoAir, FieldElement, ProofOptions, Prover, PublicInputs, RapTraceTable, Trace,
    TRACE_WIDTH,
};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Mutex;

// CAIRO PROVER
// ================================================================================================

pub struct CairoProver {
    options: ProofOptions,
    bytecode: Vec<BaseElement>,
    register_values: Vec<BaseElement>,
}

impl CairoProver {
    pub fn new(
        options: ProofOptions,
        bytecode: Vec<BaseElement>,
        register_values: Vec<BaseElement>,
    ) -> Self {
        Self {
            options,
            bytecode,
            register_values,
        }
    }

    /// Builds an execution trace for a Cairo program from the provided file.
    pub fn build_trace_from_file(&self, trace_file_path: &String) -> RapTraceTable<BaseElement> {
        let file = File::open(trace_file_path).expect("Cannot open the file.");

        let reader = Mutex::new(BufReader::new(file));

        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let real_length = usize::from_str(&line).unwrap();
        let padded_length = real_length.next_power_of_two();

        let mut trace = RapTraceTable::new(TRACE_WIDTH, padded_length);

        trace.fill(
            |state| {
                let mut line = String::new();
                line.clear();
                reader.lock().unwrap().read_line(&mut line).unwrap();
                line.pop();
                state.copy_from_slice(
                    &mut line
                        .split([','].as_ref())
                        .map(|a| BaseElement::new(u128::from_str(&a).unwrap()))
                        .collect::<Vec<BaseElement>>()[..TRACE_WIDTH],
                );
            },
            |_, state| {
                let mut line = String::new();
                line.clear();
                reader.lock().unwrap().read_line(&mut line).unwrap();
                line.pop();
                state.copy_from_slice(
                    &mut line
                        .split([','].as_ref())
                        .map(|a| BaseElement::new(u128::from_str(&a).unwrap()))
                        .collect::<Vec<BaseElement>>()[..TRACE_WIDTH],
                );
            },
        );

        trace
    }
}

impl Prover for CairoProver {
    type BaseField = BaseElement;
    type Air = CairoAir;
    type Trace = RapTraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        PublicInputs {
            bytecode: self.bytecode.clone(),
            register_values: self.register_values.clone(),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
