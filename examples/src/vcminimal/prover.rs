// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, VCMinimalAir, FieldElement, ProofOptions, Prover, PublicInputs, TraceTable,
};

use crate::utils::print_trace;

// FIBONACCI PROVER
// ================================================================================================

const N_COLS: usize = 4;

pub struct VCMinimalProver {
    options: ProofOptions,
    inputs: [u128;2],
}

impl VCMinimalProver {
    pub fn new(options: ProofOptions, inputs: [u128;2]) -> Self {
        Self { options, inputs}
    }

    pub fn build_trace(&self, sequence_length: usize) -> TraceTable<BaseElement> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        let a = BaseElement::from(self.inputs[0]);
        let b = BaseElement::from(self.inputs[1]);
        let two = BaseElement::from(2u128);

        //ALEX: modify for virtual trace
        //let mut trace = TraceTable::new_virtual(N_COLS, 8, 3);
        let mut trace = TraceTable::new(N_COLS, sequence_length);

        //ALEX: modify to not need hardcoded inputs
        trace.fill(
            |state| {
                state[0] = a;
                state[1] = b;
                state[2] = BaseElement::ZERO;
                state[3] = BaseElement::ZERO;
            },
            |_, state| {
                let a = state[0];
                let b = state[1];
                state[0] = a * a;
                state[1] = b * b;
                state[2] = two * a * b;
                state[3] = (a + b) * (a + b);
            }
        );
        print_trace(&trace, 1, 0, 0..4);
        trace
    }
}

impl Prover for VCMinimalProver {
    type BaseField = BaseElement;
    type Air = VCMinimalAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        PublicInputs {
            input: [trace.get(0, 0), trace.get(1, 0)],
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}