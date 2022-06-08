// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, CairoAir, FieldElement, ProofOptions, Prover, Trace, TraceTable, TRACE_WIDTH,
};

use std::fs::File;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::str::FromStr;


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
/*        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );*/

        let file = File::open(trace_file_path).expect("Cannot open the file.");

        let reader = BufReader::new(file);

        let mut trace = TraceTable::new(TRACE_WIDTH, 2);

        let mut line = String::new();

        trace.fill(
            |state| {
                for i in 0..33 {
                    reader.read_line(&mut line);
                    let x = u64::from_str(&line).unwrap();
                    state[i] = BaseElement::new(x.into());
                }
            },
            |_, state| {
                for i in 0..33 {
                    reader.read_line(&mut line);
                    let x = u64::from_str(&line).unwrap();
                    state[i] = BaseElement::new(x.into());
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BaseElement {
        let last_step = trace.length() - 1;
        trace.get(1, last_step)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
