// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, VCMinimalAir, FieldElement, ProofOptions, Prover, PublicInputs, TraceTable,
};

use crate::utils::print_trace;

// Virtual minimal example prover
// ================================================================================================

const N_COLS: usize = 2;

pub struct VCMinimalProver {
    options: ProofOptions,
    inputs: u128,
}

impl VCMinimalProver {
    pub fn new(options: ProofOptions, inputs: u128) -> Self {
        Self { options, inputs}
    }

    pub fn build_trace(&self, sequence_length: usize) -> TraceTable<BaseElement> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        let a = BaseElement::from(self.inputs);

        //ALEX: modify for virtual trace
        let mut trace = TraceTable::new_virtual(N_COLS, sequence_length, 1);
        //let mut trace = TraceTable::new(N_COLS, sequence_length);

        trace.fill(
            |state| {
                state[0] = a;
                state[1] = a*a;
            },
            |_, state| {
                let a = state[1];
                state[0] = a * a;
                state[1] = state[0]*state[0];
            }
        );
        print_trace(&trace, 1, 0, 0..2);
        trace
    }
}

impl Prover for VCMinimalProver {
    type BaseField = BaseElement;
    type Air = VCMinimalAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        PublicInputs {
            input: [trace.get(0, 0)],
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}