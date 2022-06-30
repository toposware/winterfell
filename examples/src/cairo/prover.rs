// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, CairoAir, FieldElement, ProofOptions, Prover, Trace, TraceTable, TRACE_WIDTH,
};

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
use std::str::FromStr;
use std::sync::{Mutex};

// CAIRO PROVER
// ================================================================================================

pub struct CairoProver {
    options: ProofOptions,
}

impl CairoProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    /// Builds an execution trace for computing a Fibonacci sequence of the specified length such
    /// that each row advances the sequence by 2 terms.
    pub fn build_trace_from_file(&self, trace_file_path: &String) -> TraceTable<BaseElement> {

        let file = File::open(trace_file_path).expect("Cannot open the file.");

        let reader = Mutex::new(BufReader::new(file));

        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let length = usize::from_str(&line).unwrap();

        assert!(
            length.is_power_of_two(),
            "program length must be a power of 2"
        );

        let mut trace = TraceTable::new(TRACE_WIDTH, length);

        trace.fill(
            |state| {
                let mut line = String::new();
                for i in 0..TRACE_WIDTH {
                    line.clear();
                    reader.lock().unwrap().read_line(&mut line).unwrap();
                    line.pop();
                    let x = u128::from_str(&line).unwrap();
                    state[i] = BaseElement::new(x.into());
                }
            },
            |row, state| {
                let mut line = String::new();
                for i in 0..TRACE_WIDTH {
                    line.clear();
                    reader.lock().unwrap().read_line(&mut line).unwrap();
                    line.pop();
                    println!("{:?} - {:?} - {:?}", row, i, &line);
                    let x = u128::from_str(&line).unwrap();
                    state[i] = BaseElement::new(x.into());
                }

                // TODO: would need dynamic checking to turn the last row into garbage
                // or add extra ones if needed
                if row == length - 2 {
                    state.copy_from_slice(&mut rand_utils::rand_array::<BaseElement, TRACE_WIDTH>());
                }
            },
        );

        trace
    }
}

impl Prover for CairoProver {
    type BaseField = BaseElement;
    type Air = CairoAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> () {
        ()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
