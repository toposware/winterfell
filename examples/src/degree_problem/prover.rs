// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, DegreeProblemAir, FieldElement, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};

use crate::utils::print_trace;

// FIBONACCI PROVER
// ================================================================================================

const N_COLS: usize = 33;

pub struct DegreeProblemProver {
    options: ProofOptions,
}

impl DegreeProblemProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        // lsb is the rightmost bit
        let trace= TraceTable::init(
            vec![
                vec![ 0, 0, 0, 0, 0, 0, 0, 0],
                vec![ 0, 0, 0, 0, 0, 0, 0, 1],
                vec![ 0, 0, 0, 0, 0, 0, 1, 1],
                vec![ 0, 0, 0, 0, 0, 1, 1, 1],
            ].into_iter().map(
                |row| row.into_iter().map(
                    |value| BaseElement::from(value as u128)
                ).collect()
            ).collect()
        );
        print_trace(&trace, 1, 0, 0..4);
        trace
    }
}

impl Prover for DegreeProblemProver {
    type BaseField = BaseElement;
    type Air = DegreeProblemAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> PublicInputs {
        PublicInputs{}
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}