// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, ProofOptions, Prover, PublicInputs, RangeAir, TraceTable};

use super::air::PERIOD;
// use crate::utils::print_trace;
use rand_utils::rand_value;

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

        // PERIOD different range checks
        let mut trace = TraceTable::new(3, 2 * PERIOD * sequence_length);

        // we put an artificial assertion stating that the value in the first 2 columns
        // of the first row should be 0 and the value in the third row should be 1

        // column 0 should contain values in the range i%PERIOD+0, i%PERIOD+1 in step i
        // column 1 should contain bits everywhere except on i%(PERIOD/2-42) where it contains 42 or 43
        // column 2 should contain the sum of previous and next row everywhere except
        //          i%2PERIOD+42 where it contains the product (first row is 1)
        trace.fill(
            |state| {
                state[0] = BaseElement::new(0);
                state[1] = BaseElement::new(0);
                state[2] = BaseElement::new(1);
            },
            |i, state| {
                let step_mod_period: u128 = (i as u128 + 1) % (PERIOD as u128);
                let step_mod_half_period: u128 = (i as u128 + 1) % (PERIOD as u128 / 2);
                let step_mod_double_period: u128 = (i as u128 + 1) % (2 * PERIOD as u128);

                // column 0. We also keep its current value
                let bit = rand_value::<u128>() % 2;
                let current = state[0];
                state[0] = BaseElement::new(step_mod_period + bit);

                // column 1
                let bit = rand_value::<u128>() % 2;
                if step_mod_half_period != (PERIOD as u128 / 2) - 42 {
                    state[1] = BaseElement::new(bit);
                } else {
                    state[1] = BaseElement::new(42 + bit);
                }

                // column 3
                if step_mod_double_period == 42 {
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

        PublicInputs { input_value }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
