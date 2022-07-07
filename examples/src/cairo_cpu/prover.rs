// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, CairoCpuAir, FieldElement, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};

use crate::utils::print_trace;

// FIBONACCI PROVER
// ================================================================================================

const N_COLS: usize = 33;

pub struct CairoCpuProver {
    options: ProofOptions,
}

impl CairoCpuProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // assert!(
        //     sequence_length.is_power_of_two(),
        //     "sequence length must be a power of 2"
        // );
        // lsb is the rightmost bit
        let f: [[u128; 15]; 8] =
        //                          These are the relevant flags
        //                                |       |          
        [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
            [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
            [0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0],
            [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
            [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0],
            [1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0]
        ];
        let res = [
            BaseElement::from(1u128), BaseElement::from(2u128), 
            BaseElement::from(3u128), BaseElement::from(4u128), 
            BaseElement::from(5u128), BaseElement::from(6u128),
            BaseElement::from(7u128), BaseElement::from(8u128)];
        
        let f_tilde: Vec<_> = f.iter().map(
            |f_row| {
                let (_, f_tilde_row) = f_row.iter().rev()
                    .fold(
                        (0, vec![BaseElement::ZERO]),
                        |(old_val, mut old_vec), bit| {
                            let new_val = (old_val << 1) + bit;
                            let mut new_vec = vec![BaseElement::from(new_val)];
                            new_vec.append(&mut old_vec);
                            (new_val, new_vec)
                        }
                    );
                f_tilde_row
            }
        ).collect();
        let ap_column = get_ap_column(&f, BaseElement::ONE, &res);

        let mut trace = TraceTable::new_virtual(N_COLS, 8, 33);
        trace.fill(
            |state| {
                state[0] = ap_column[0];
                state[1..3].copy_from_slice(&[BaseElement::ZERO; 2]);
                state[3..19].copy_from_slice(&f_tilde[0]);
                state[19..].copy_from_slice(&[BaseElement::ZERO; 14]);
                state[32] = res[0];
            },
            |step, state| {
                state[0] = ap_column[step + 1];
                state[1..3].copy_from_slice(&[BaseElement::ZERO; 2]);
                state[3..19].copy_from_slice(&f_tilde[step + 1]);
                state[19..].copy_from_slice(&[BaseElement::ZERO; 14]);
                state[32] = res[step + 1];
            }
        );
        print_trace(&trace, 1, 0, 0..33);
        trace
    }
}

impl Prover for CairoCpuProver {
    type BaseField = BaseElement;
    type Air = CairoCpuAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> PublicInputs {
        PublicInputs{}
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

// HELPERS

/// Returns the value of ap column for `steps` steps assuming f and res are equal to
/// `f, res` respectively, and that intially ap was `ap_0`
fn get_ap_column<const STEPS: usize>(f: &[[u128; 15]; STEPS], ap_0: BaseElement, res: &[BaseElement; STEPS]) -> Vec<BaseElement> {
    (0..STEPS).fold(
        vec![ap_0],
        |mut acc, i| {
            acc.push(
                acc.last().copied().unwrap()
                + BaseElement::from(f[i][10])*res[i]
                + BaseElement::from(f[i][11]) 
                + BaseElement::from(f[i][12])*BaseElement::from(2 as u128)
            );
            acc
        }
    )
}
