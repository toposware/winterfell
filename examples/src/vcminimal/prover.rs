// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{math::{FieldElement, StarkField}, crypto::ElementHasher};

use super::{
    BaseElement, VCMinimalAir, ProofOptions, Prover, PublicInputs, TraceTable,
};

use crate::utils::print_trace;

use std::panic;

// Virtual minimal example prover
// ================================================================================================

pub struct VCMinimalProver {
    options: ProofOptions,
    input: u128,
}

impl VCMinimalProver {
    pub fn new(options: ProofOptions, input: u128) -> Self {
        Self { options, input}
    }

    pub fn build_trace(&self, sequence_length: usize, width: usize, real_width: usize) 
    -> TraceTable<BaseElement>
    {
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
                for i in 0..width {
                    state[i] = power_of_two_exp(input, i + 1);
                }
            },
            |step, state| {
                for i in 0..width {
                    let log_power = width*(step + 1) + i + 1;
                    state[i] = power_of_two_exp(input, log_power);
                }
            }
        );
        print_trace(&trace, 1, 0, 0..5);
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

/// Computes x.exp(2.pow(n))
fn power_of_two_exp(x: BaseElement, n:usize) -> BaseElement {
    if n == 0 {
        x
    } else {
        power_of_two_exp(x*x, n-1)
    }
}