// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    PedersenHashAir, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};

use super::{ecc, field, hash::get_constant_points};
use winterfell::TraceTableFragment;
use winterfell::math::{curves::curve_f63::Scalar, fields::f63::BaseElement, FieldElement};
use crate::utils::ecc::{compute_slope, compute_add_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH};
use crate::utils::print_trace_63;

// FIBONACCI PROVER
// ================================================================================================

const N_COLS: usize = AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH;

pub struct PedersenHashProver {
    options: ProofOptions,
}

impl PedersenHashProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // Some arbitrary binary decompositions of field elements
        let bits: [[u64; 15]; 8] =
        [
           // [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
            [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0],
            [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
            [0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0],
            [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
            [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0],
            [1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0]
        ];
        let prefixes = get_prefixes(&bits);

        let mut trace = TraceTable::new(N_COLS, 64);
        let pedersen_constant_points = get_constant_points();
        update_trace_rows(&mut trace, bits, prefixes, pedersen_constant_points);
        print_trace_63(&trace, 1, 0, 0..33);
        trace
    }
}

impl Prover for PedersenHashProver {
    type BaseField = BaseElement;
    type Air = PedersenHashAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> PublicInputs {
        PublicInputs{}
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

// HELPERS

fn get_prefixes(bits: &[[u64; 15]; 8]) -> Vec<Vec<BaseElement>> {
    bits.iter().map(
        |bits_row| {
            let (_, bits_tilde_row) = bits_row.iter().rev()
                .fold(
                    (0, vec![BaseElement::ZERO]),
                    |(old_val, mut old_vec), bit| {
                        let new_val = (old_val << 1) + bit;
                        let mut new_vec = vec![BaseElement::from(new_val)];
                        new_vec.append(&mut old_vec);
                        (new_val, new_vec)
                    }
                );
            bits_tilde_row
        }
    ).collect()
}

fn update_trace_rows(
    trace: &mut TraceTable<BaseElement>,
    bits: [[u64; 15]; 8], 
    prefixes: Vec<Vec<BaseElement>>, 
    constant_points: [[BaseElement; AFFINE_POINT_WIDTH]; 16])
{
    let result = vec![vec![BaseElement::ZERO; AFFINE_POINT_WIDTH]];
    let mut state = [BaseElement::ZERO; N_COLS];
    state[0..AFFINE_POINT_WIDTH].copy_from_slice(&constant_points[0]);
    state[AFFINE_POINT_WIDTH] = prefixes[0][0];

    let point1 = &mut [BaseElement::ZERO; AFFINE_POINT_WIDTH];
    point1.copy_from_slice(&state[0..AFFINE_POINT_WIDTH]);
    let point2 = &constant_points[0];
    compute_slope(&mut state, point1, point2);
    trace.update_row(0, &state);

    for (i, (bits, prefixes)) in bits.iter().zip(prefixes.iter()).enumerate()
    {
        for (j, (&bit, &prefix)) in  bits.iter().zip(prefixes.iter()).enumerate() {
            state[AFFINE_POINT_WIDTH] = prefix;
            let point1 = &mut [BaseElement::ZERO; AFFINE_POINT_WIDTH];
            point1.copy_from_slice(&state[0..AFFINE_POINT_WIDTH]);
            let point2 = &constant_points[8*i + j];
            compute_slope(
                &mut state[AFFINE_POINT_WIDTH + 1..AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH],
                point1,
                point2
            );
            if bit == 1u64 {
                let point = &constant_points[0];
                let slope = &mut [BaseElement::ZERO; POINT_COORDINATE_WIDTH];
                slope.copy_from_slice(&state[AFFINE_POINT_WIDTH + 1..AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH]);
                compute_add_affine(&mut state[0..AFFINE_POINT_WIDTH], point, slope);
            }
            trace.update_row(8*i + j, &state);
        }
    }
}

