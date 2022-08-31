// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    PedersenHashAir, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};

use std::ops::Range;

use super::hash::{get_intial_constant_point, get_constant_points};
use winterfell::math::{fields::f63::BaseElement, FieldElement};
use crate::utils::ecc::{compute_slope, compute_add_affine_with_slope, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH};
use crate::utils::print_trace_63;

// PEDERSEN HASH PROVER
// ================================================================================================
const N_CHUNKS: usize = 8;
pub const BITS_PER_CHUNK: usize = 63;
const N_BITS: usize = N_CHUNKS*BITS_PER_CHUNK;
pub const CYCLE_LENGTH: usize = 64;
pub const TRACE_LENGTH: usize = CYCLE_LENGTH*(N_CHUNKS.next_power_of_two());
pub const TRACE_WIDTH: usize = AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH;

// Constants for reading from the state
const CURVE_POINT: Range<usize> = 0..AFFINE_POINT_WIDTH;
const PREFIX: usize = AFFINE_POINT_WIDTH;
const SLOPE: Range<usize> = AFFINE_POINT_WIDTH + 1 .. AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1;

const EXAMPLE: [u64; N_CHUNKS] = [
    2u64.pow(63) - 1,
    2u64.pow(63) - 2,
    2u64.pow(63) - 3,
    2u64.pow(63) - 4,
    2u64.pow(63) - 5,
    2u64.pow(63) - 6,
    2u64.pow(63) - 1000,
    2u64.pow(63) - 8
];

// const _TRIVIAL_EXAMPLE: [[u64; BITS_PER_CHUNK]; N_CHUNKS] = [[0u64; BITS_PER_CHUNK]; N_CHUNKS];

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
        let inputs = EXAMPLE;

        let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
        let initial_pedersen_point = get_intial_constant_point();
        let pedersen_constant_points = get_constant_points();
        update_with_subset_sum(&mut trace, inputs, initial_pedersen_point, pedersen_constant_points);
        print_trace_63(&trace, 1, 0, 11..13);
        trace
    }
}

impl Prover for PedersenHashProver {
    type BaseField = BaseElement;
    type Air = PedersenHashAir<TRACE_LENGTH, CYCLE_LENGTH, 1>;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
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
    inputs: [u64; N_CHUNKS],
    initial_constant_point: [BaseElement; AFFINE_POINT_WIDTH], 
    constant_points: [[BaseElement; AFFINE_POINT_WIDTH]; TRACE_LENGTH])
{
    
    let mut state = [BaseElement::ZERO; TRACE_WIDTH];
    state[CURVE_POINT].copy_from_slice(&initial_constant_point);

    for i in 0..N_CHUNKS {

        // update the first row for this chunk
        let current_row_index = i*CYCLE_LENGTH;
        let mut point = [BaseElement::ZERO; AFFINE_POINT_WIDTH];
        point.copy_from_slice(&state[CURVE_POINT]);
        compute_slope(
            &mut state[SLOPE],
            &point,
            &constant_points[current_row_index]
        );
        
        let mut prefix = inputs[i];
        state[PREFIX] = BaseElement::from(prefix);
        trace.update_row(i*CYCLE_LENGTH, &state);

        for (current, next) in (0..CYCLE_LENGTH-1).zip(1..CYCLE_LENGTH) {
            let current_row_index = i*CYCLE_LENGTH + current;
            let next_row_index = i*CYCLE_LENGTH + next;

            state[PREFIX] = BaseElement::from(prefix >> 1);

            if prefix%2 == 1 {
                // retrieve the slope of the current row
                let slope = &mut [BaseElement::ZERO; POINT_COORDINATE_WIDTH];
                slope.copy_from_slice(&state[SLOPE]);
                compute_add_affine_with_slope(&mut state[CURVE_POINT], &constant_points[current_row_index], slope);
            }
            // compute the slope for the next row
            point.copy_from_slice(&state[CURVE_POINT]);
            compute_slope(
                &mut state[SLOPE],
                &point,
                &constant_points[next_row_index]
            );
            
            trace.update_row(next_row_index, &state);
            prefix >>= 1;
        }
    }
}

