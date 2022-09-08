// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    MuxAir, MuxPublicInputs, SubsetSumAir, ZeroAir, ProofOptions, Prover, PublicInputs, Trace, TraceTable,
};
use winterfell::Air;

use std::ops::Range;

use super::hash::{get_intial_constant_point, get_constant_points};
use winterfell::math::{fields::f63::BaseElement, FieldElement};
use crate::utils::ecc::{compute_slope, compute_add_affine_with_slope, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH};
use crate::utils::print_trace_63;

// PEDERSEN HASH PROVER
// ================================================================================================
const N_CHUNKS: usize = 1;
pub const BITS_PER_CHUNK: usize = 3;
const N_BITS: usize = N_CHUNKS*BITS_PER_CHUNK;
pub const CYCLE_LENGTH: usize = BITS_PER_CHUNK.next_power_of_two();
pub const TRACE_LENGTH: usize = CYCLE_LENGTH*(N_CHUNKS.next_power_of_two());
pub const TRACE_WIDTH: usize = AFFINE_POINT_WIDTH + 1 + POINT_COORDINATE_WIDTH;

const MUX_N_CHUNKS: usize = 8;
const MUX_TRACE_WIDTH: usize = TRACE_WIDTH*CYCLE_LENGTH/2;
pub const MUX_TRACE_LENGTH: usize = MUX_N_CHUNKS;

// Constants for reading from the state
pub const PREFIX: usize = 0;
pub const CURVE_POINT: Range<usize> = 1..AFFINE_POINT_WIDTH + 1;
pub const SLOPE: Range<usize> = AFFINE_POINT_WIDTH + 1 .. AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1;

// Work around for the "more qualified paths" issue
type Type<T> = T;

const EXAMPLE: [u64; N_CHUNKS] = [
    4u64,
];

const MUX_EXAMPLE: [u64; MUX_N_CHUNKS] = [1u64; MUX_N_CHUNKS];

pub struct SubsetSumProver {
    options: ProofOptions,
}

impl SubsetSumProver {

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
        print_trace_63(&trace, 1, 0, 0..2);
        print_trace_63(&trace, 1, 0, SLOPE);
        trace
    }
}

impl Prover for SubsetSumProver {
    type BaseField = BaseElement;
    type Air = SubsetSumAir<4,4> ;
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
        let mut prefix = inputs[i];
        if prefix%2 == 1 {
            compute_slope(
                &mut state[SLOPE],
                &point,
                &constant_points[current_row_index]
            );
        }

        state[PREFIX] = BaseElement::from(prefix);
        trace.update_row(i*CYCLE_LENGTH, &state);

        for (current, next) in (0..CYCLE_LENGTH-1).zip(1..CYCLE_LENGTH) {
            let current_row_index = i*CYCLE_LENGTH + current;
            let next_row_index = i*CYCLE_LENGTH + next;
            state[PREFIX] = BaseElement::from(prefix >> 1);

            let slope = &mut [BaseElement::ZERO; POINT_COORDINATE_WIDTH];
            if prefix%2 == 1 {
                // retrieve the slope of the current row
                slope.copy_from_slice(&state[SLOPE]);
                compute_add_affine_with_slope(&mut state[CURVE_POINT], &constant_points[current_row_index], slope);

            }
            // Compute the slope for the next row
            prefix >>= 1;
            if prefix%2 == 1 {
                point.copy_from_slice(&state[CURVE_POINT]);
                compute_slope(
                    &mut state[SLOPE],
                    &point,
                    &constant_points[next_row_index]
                );
            } else {
                state[SLOPE].copy_from_slice(&[BaseElement::ZERO; POINT_COORDINATE_WIDTH]);
            }
            
            trace.update_row(next_row_index, &state);
        }
    }
}

pub struct MuxProver {
    options: ProofOptions,
}

impl MuxProver {

    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }


    // Trace layou
    //
    pub fn build_trace(&self) -> TraceTable<BaseElement> {
        // Some arbitrary binary decompositions of field elements
        let inputs = MUX_EXAMPLE;

        let mut trace = TraceTable::new(MUX_TRACE_WIDTH, MUX_TRACE_LENGTH);
        let initial_pedersen_point = get_intial_constant_point();
        let pedersen_constant_points = get_constant_points();
        update_with_mux_subset_sum(&mut trace, inputs, initial_pedersen_point, pedersen_constant_points);
        print_trace_63(&trace, 1, 0, 0..2);
        print_trace_63(&trace, 1, 0, SLOPE);
        print_trace_63(&trace, 1, 0, TRACE_WIDTH..TRACE_WIDTH + 2);
        trace
    }
}

impl Prover for MuxProver {
    type BaseField = BaseElement;
    type Air = MuxAir<SubsetSumAir<32,32>, ZeroAir, 4>;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MuxPublicInputs<SubsetSumAir<32,32>, ZeroAir> {

        MuxPublicInputs {
            input_left: Type::<<SubsetSumAir<4,4> as Air>::PublicInputs> {},
            input_right: Type::<<ZeroAir as Air>::PublicInputs> {}
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

fn update_with_mux_subset_sum(
    trace: &mut TraceTable<BaseElement>,
    inputs: [u64; MUX_N_CHUNKS],
    initial_constant_point: [BaseElement; AFFINE_POINT_WIDTH], 
    constant_points: [[BaseElement; AFFINE_POINT_WIDTH]; MUX_TRACE_LENGTH])
{
    
    let mut state = [BaseElement::ZERO; MUX_TRACE_WIDTH];
    state[CURVE_POINT].copy_from_slice(&initial_constant_point);

    for i in 0..N_CHUNKS {
        let mut prefix = inputs[i];
        for j in 0..2 {
            state[PREFIX] = BaseElement::from(prefix);
            let mut point = [BaseElement::ZERO; AFFINE_POINT_WIDTH];
            point.copy_from_slice(&state[CURVE_POINT]);
            if prefix%2 == 1 {
                compute_slope(
                    &mut state[SLOPE],
                    &point,
                    &constant_points[i*CYCLE_LENGTH]
                );
            } else {
                state[SLOPE].copy_from_slice(&[BaseElement::ZERO; POINT_COORDINATE_WIDTH]);
            }
            for (current, next) in (0..CYCLE_LENGTH/2-1).zip(1..CYCLE_LENGTH/2) {
                let current_row_index = i*CYCLE_LENGTH + current;
                let next_row_index = i*CYCLE_LENGTH + next;

                let mut new_point = [BaseElement::ZERO; AFFINE_POINT_WIDTH];
                new_point.copy_from_slice(&state[CURVE_POINT]);
                if prefix%2 == 1 {
                    // compute the new point
                    compute_add_affine_with_slope(
                        &mut new_point, 
                        &constant_points[current_row_index], 
                        &state[SLOPE]
                    );
                }
                let state = &mut state[next*TRACE_WIDTH..];

                // compute the slope for the next row
                prefix >>= 1;
                state[PREFIX] = BaseElement::from(prefix);
                state[CURVE_POINT].copy_from_slice(&new_point);
                if prefix%2 == 1 {
                    compute_slope(
                        &mut state[SLOPE],
                        &new_point,
                        &constant_points[next_row_index]
                    );
                }
            }
            trace.update_row(2*i + j, &state);
        }          
    }
}