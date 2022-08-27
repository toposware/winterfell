// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    PedersenHashAir, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};

use std::ops::Range;

use super::{hash::get_constant_points};
use winterfell::math::{fields::f63::BaseElement, FieldElement};
use crate::utils::ecc::{compute_slope, compute_add_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH};
use crate::utils::print_trace_63;

// PEDERSEN HASH PROVER
// ================================================================================================
const N_CHUNKS: usize = 8;
pub const BITS_PER_CHUNK: usize = 15;
const N_BITS: usize = N_CHUNKS*BITS_PER_CHUNK;
pub const CYCLE_LENGTH: usize = BITS_PER_CHUNK.next_power_of_two();
pub const TRACE_LENGTH: usize = (N_BITS as u32).next_power_of_two() as usize;
pub const N_CYCLES: usize = TRACE_LENGTH/CYCLE_LENGTH;
pub const TRACE_WIDTH: usize = AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH;

// Constants for reading from the state
const CURVE_POINT: Range<usize> = 0..AFFINE_POINT_WIDTH;
const PREFIX: usize = AFFINE_POINT_WIDTH;
const SLOPE: Range<usize> = AFFINE_POINT_WIDTH + 1 .. AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1;

const EXAMPLE: [[u64; BITS_PER_CHUNK]; N_CHUNKS] =
    [
        [1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0],
        [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
        [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0],
        [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0],
        [0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0],
        [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
        [0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0],
        [0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0],
    ];

const _TRIVIAL_EXAMPLE: [[u64; BITS_PER_CHUNK]; N_CHUNKS] = [[0u64; BITS_PER_CHUNK]; N_CHUNKS];

pub struct PedersenHashProver {
    options: ProofOptions,
}

impl PedersenHashProver {

    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }


    // Trace layou
    //
    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // Some arbitrary binary decompositions of field elements
        let bits  = EXAMPLE;

        let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
        let pedersen_constant_points = get_constant_points();
        update_with_subset_sum(&mut trace, bits, pedersen_constant_points);
        print_trace_63(&trace, 1, 0, 11..13);
        trace
    }
}

impl Prover for PedersenHashProver {
    type BaseField = BaseElement;
    type Air = PedersenHashAir<TRACE_LENGTH, 1>;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> PublicInputs {
        PublicInputs{}
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

// HELPERS

fn get_prefixes(bits: &[[u64; BITS_PER_CHUNK]; N_CHUNKS]) -> Vec<Vec<BaseElement>> {
    let prefixes = bits.iter()
        .fold(
            vec![],
            |mut old_vec, bits_row|
            {
                let new_vec = bits_row.iter()
                    .fold(
                        vec![0u64],
                        | mut old_vec, bit| {
                            let new_val = (old_vec[0] << 1) + bit;
                            let mut new_vec = vec![new_val];
                            new_vec.append(&mut old_vec);
                            new_vec
                        }
                    );
                old_vec.push(new_vec);
                old_vec
            }
        );
    let prefixes: Vec<Vec<_>> = prefixes.into_iter()
    .map(
        |row| row.iter().map(|&value| BaseElement::from(value)).collect()
    ).collect();
    for prefixes in prefixes.iter() {
        for (&current, &next) in prefixes.iter().zip(prefixes.iter().skip(1)) {
            let bit  = current - (BaseElement::ONE + BaseElement::ONE)*next;
            assert_eq!(bit*(bit - BaseElement::ONE), BaseElement::ZERO);
        }
    }
    prefixes
}

fn update_with_subset_sum(
    trace: &mut TraceTable<BaseElement>,
    bits: [[u64; BITS_PER_CHUNK]; N_CHUNKS],
    constant_points: [[BaseElement; AFFINE_POINT_WIDTH]; TRACE_LENGTH + 1])
{
    let prefixes = get_prefixes(&bits);
    
    let mut state = [BaseElement::ZERO; TRACE_WIDTH];
    state[CURVE_POINT].copy_from_slice(&constant_points[0]);

    for i in 0..N_CYCLES {

        // update the first row for this chunk
        let current_row_index = i*CYCLE_LENGTH;
        let mut point = [BaseElement::ZERO; AFFINE_POINT_WIDTH];
        point.copy_from_slice(&state[CURVE_POINT]);
        compute_slope(
            &mut state[SLOPE],
            &point,
            &constant_points[current_row_index + 1]
        );
        state[PREFIX] = prefixes[i][0];
        trace.update_row(i*CYCLE_LENGTH, &state);

        for (current, next) in (0..CYCLE_LENGTH-1).zip(1..CYCLE_LENGTH) {
            let next_row_index = i*CYCLE_LENGTH + next;

            state[PREFIX] = prefixes[i][next];

            // bits are in backward order
            if bits[i][BITS_PER_CHUNK - 1 - current] == 1u64 {
                // retrieve the slope of the current row
                let slope = &mut [BaseElement::ZERO; POINT_COORDINATE_WIDTH];
                slope.copy_from_slice(&state[SLOPE]);
                compute_add_affine(&mut state[CURVE_POINT], &constant_points[next_row_index], slope);
            }
            // compute the slope for the next row
            point.copy_from_slice(&state[CURVE_POINT]);
            compute_slope(
                &mut state[SLOPE],
                &point,
                &constant_points[next_row_index + 1]
            );
            trace.update_row(next_row_index, &state);
        }
    }
}
