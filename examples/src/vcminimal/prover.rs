// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::math::FieldElement;

use super::{BaseElement, ProofOptions, Prover, PublicInputs, TraceTable, VCMinimalAir};

// use crate::utils::print_trace;

// Virtual minimal example prover
// ================================================================================================

pub struct VCMinimalProver {
    options: ProofOptions,
    input: u128,
}

impl VCMinimalProver {
    pub fn new(options: ProofOptions, input: u128) -> Self {
        Self { options, input }
    }

    pub fn build_trace(
        &self,
        sequence_length: usize,
        width: usize,
        real_width: usize,
    ) -> TraceTable<BaseElement> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        let mut trace = TraceTable::new_virtual(width, sequence_length, real_width);
        //ALEX: modify for virtual trace
        //let mut trace = TraceTable::new(N_COLS, sequence_length);

        let input = BaseElement::from(self.input);
        trace.fill(
            |state| {
                state[0] = BaseElement::ONE;
                for i in 1..width {
                    state[i] = state[i - 1] * input;
                }
            },
            |_, state| {
                for i in 0..width {
                    state[i] *= state[width - 1];
                }
            },
        );
        //print_trace(&trace, 1, 0, 0..5);
        trace
    }
}

impl Prover for VCMinimalProver {
    type BaseField = BaseElement;
    type Air = VCMinimalAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        PublicInputs {
            input: trace.get(0, 0),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
