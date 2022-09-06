// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, ProofOptions, Prover, PublicInputs, RangeAir, Trace, TraceTable};

use super::air::PERIOD;
use rand_utils::rand_value;
// use crate::utils::print_trace;

// COLLATZ PROVER
// ================================================================================================

pub struct RangeProver {
    options: ProofOptions,
}

impl RangeProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    /// Builds an execution trace for computing a Collatz sequence.
    pub fn build_trace(&self, sequence_length: usize) -> TraceTable<BaseElement> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // 32 different range checks
        let mut trace = TraceTable::new(3, 1024 * sequence_length);

        // we put an artificial assertion stating that the value in the first 2 columns
        // of the first row should be 0

        // column 0 should contain values in the range i%32+0, i%32+1 in step i
        // column 1 should contain bits everywhere except on i%512-42 where it contains 42 or 43
        // column 2 should contain values the sum of previous and next row everywhere except
        //          i%1024+42 where it contains the product (first row is 1)
        trace.fill(
            |state| {
                state[0] = BaseElement::new(0);
                state[1] = BaseElement::new(0);
                state[2] = BaseElement::new(1);
            },
            |i, state| {
                let step_mod_period: u128 = (i as u128 + 1) % (PERIOD as u128);
                let step_mod_512: u128 = (i as u128 + 1) % 512;
                let step_mod_1024: u128 = (i as u128 + 1) % 1024;

                // column 0. We also keep its current value
                let bit = rand_value::<u128>() % 2;
                let current = state[0];
                state[0] = BaseElement::new(step_mod_period + bit);

                // column 1
                let bit = rand_value::<u128>() % 2;
                if step_mod_512 != 512 - 42 {
                    state[1] = BaseElement::new(bit);
                } else {
                    state[1] = BaseElement::new(42 + bit);
                }

                // column 3
                if step_mod_1024 == 42 {
                    state[2] = state[0] * current;
                } else {
                    state[2] = state[0] + current;
                }
            },
        );

        // print_trace(&trace, 1, 0, 0..3);
        trace
    }
}

impl Prover for RangeProver {
    type BaseField = BaseElement;
    type Air = RangeAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let input_value = trace.get(0, 0);
        let sequence_length = trace.length() / 1024;

        PublicInputs {
            input_value,
            sequence_length,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
