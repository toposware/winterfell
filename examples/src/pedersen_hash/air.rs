// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, FieldElement, ProofOptions, prover::{TRACE_WIDTH, PREFIX, CURVE_POINT, SLOPE},
    hash::{get_intial_constant_point, get_constant_points}, ecc::GENERATOR};
use crate::{utils::ecc::{enforce_point_addition_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH}, pedersen_hash::prover::{MUX_TRACE_LENGTH}};
use cheetah::group::ff::Field;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree
};

use std::{cmp::min, result};

pub struct PublicInputs {
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
    }
}

pub trait ComposableAir: Air {
    const NUM_READ_CELLS_CURRENT: usize;
    const NUM_READ_CELLS_NEXT: usize;
}

pub struct SubsetSumAir<const CYCLE_LENGTH: usize, const CONSTANTS_CYCLE_LENGTH: usize> {
    context: AirContext<<SubsetSumAir<CYCLE_LENGTH, CONSTANTS_CYCLE_LENGTH> as Air>::BaseField>,
}

impl<const CYCLE_LENGTH: usize, const CONSTANTS_CYCLE_LENGTH: usize>
SubsetSumAir<CYCLE_LENGTH, CONSTANTS_CYCLE_LENGTH> {
    const INITAL_CONSTANT_POINT: [BaseElement; AFFINE_POINT_WIDTH] = get_intial_constant_point();
}

impl<const TRACE_LENGTH: usize, const CONSTANTS_CYCLE_LENGTH: usize>
ComposableAir for SubsetSumAir<TRACE_LENGTH, CONSTANTS_CYCLE_LENGTH> {
    const NUM_READ_CELLS_CURRENT: usize = TRACE_WIDTH;
    const NUM_READ_CELLS_NEXT: usize = TRACE_WIDTH;
}

impl<const TRACE_LENGTH: usize, const CONSTANTS_CYCLE_LENGTH: usize>
Air for SubsetSumAir<TRACE_LENGTH, CONSTANTS_CYCLE_LENGTH> {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        // This checks could be performed at compile time
        assert!(TRACE_LENGTH.is_power_of_two() && CONSTANTS_CYCLE_LENGTH.is_power_of_two());

        assert!(TRACE_LENGTH >= CONSTANTS_CYCLE_LENGTH);
        let mut degrees = vec![TransitionConstraintDegree::new(2)];
        degrees.append(&mut vec![TransitionConstraintDegree::new(2); 6]);
        degrees.append(&mut vec![TransitionConstraintDegree::new(2); 12]); // TODO: Check

        let context =
        // Why does Air context require at least 1 assertion?
        AirContext::new(
            trace_info, 
            degrees,
            12,
            options);
        SubsetSumAir {
            context,
        }
    }
    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let bit = current[PREFIX] - (E::ONE + E::ONE)*next[PREFIX];
        let lhs = &current[CURVE_POINT];
        let rhs = &periodic_values[..AFFINE_POINT_WIDTH];
        let slope = &current[SLOPE];
        let point = &next[CURVE_POINT];

        // constraint pedersen/hash0/ec_subset_sum/booleanity_test 
        result[0] = bit*(bit - E::ONE);

        // result[0..POINT_COORDINATE_WIDTH] corresponds to constraint pedersen/hash0/ec_subset_sum/add_points/slope
        // result[POINT_COORDINATE_WIDTH .. 2*POINT_COORDINATE_WIDTH] corresponds to pedersen/hash0/ec_subset_sum/add_points/x
        // result[2*POINT_COORDINATE_WIDTH .. 3*POINT_COORDINATE_WIDTH] corresponds to pedersen/hash0/ec_subset_sum/add_points/y
        // TODO: constraints pedersen/hash0/ec_subset_sum/copy_point/x and pedersen/hash0/ec_subset_sum/copy_point/y are missing. 
        enforce_point_addition_affine(
            &mut result[1..],
            lhs,
            rhs,
            slope,
            point,
            bit);

        // Constraints pedersen/hash0/ec_subset_sum/bit_unpacking/ require virtual columns

    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let mut initial_point_assertions = Vec::new();
        for i in 0..AFFINE_POINT_WIDTH{
            initial_point_assertions.push(Assertion::single(i + 1, 0, Self::INITAL_CONSTANT_POINT[i]));
        }
        initial_point_assertions
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut periodic_columns = vec![Vec::new(); AFFINE_POINT_WIDTH];
        for point in get_constant_points::<CONSTANTS_CYCLE_LENGTH>().into_iter() {
            for (i, column) in periodic_columns.iter_mut().enumerate() {
                column.push(point[i]);
            }
        }
        periodic_columns
    }
}

pub struct ZeroAir {
    context: AirContext<BaseElement>
}

impl ComposableAir for ZeroAir {
    const NUM_READ_CELLS_CURRENT: usize = 1;
    const NUM_READ_CELLS_NEXT: usize = 0;
}

impl Air for ZeroAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let mut degrees = vec![TransitionConstraintDegree::new(1)];

        let context =
            // Why does Air context require at least 1 assertion?
            AirContext::new(
                trace_info, 
                degrees,
                1, //TODO: Should be 0
                options);
        ZeroAir {
            context,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        result[0] = frame.current()[0];
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![]
    }
}

pub struct MuxPublicInputs<A: ComposableAir, B: ComposableAir> {
    pub input_left: A::PublicInputs,
    pub input_right: B::PublicInputs
}

impl<A: ComposableAir, B: ComposableAir> Serializable for MuxPublicInputs<A, B> {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.input_left.write_into(target);
        self.input_right.write_into(target);
    }
}

pub struct MuxAir<A, B, const RATIO: usize>
where
    A: ComposableAir,
    B: ComposableAir<BaseField = A::BaseField>
{
    context: AirContext<A::BaseField>,
    air_left: A,
    air_right: B,
    width_left: usize,
    width_right: usize,
    num_transition_constraints_left: usize,
    num_transition_constraints_right: usize,
    num_periodic_values_left: usize,
    num_periodic_values_right: usize,
}

/// An Air program that executes A for RATIO - 1 steps, using the first RATIO cells of the current frame,
/// and then executes B only once using 2 extra cells of the . 
impl<A, B, const RATIO: usize> Air for MuxAir<A, B, RATIO>
where
    A: ComposableAir,
    B: ComposableAir<BaseField = A::BaseField>
{
    type BaseField = A::BaseField;
    type PublicInputs = MuxPublicInputs<A, B>;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert!(RATIO.is_power_of_two() && RATIO > 1);

        let air_left = A::new(trace_info.clone(), pub_inputs.input_left, options.clone());
        let air_right = B::new(trace_info.clone(), pub_inputs.input_right, options.clone());
        // TODO: How to add auxiliar transition degrees?
        let mut transition_constraint_degrees = vec![];
        for _  in 0..RATIO - 1 {
            transition_constraint_degrees.append(&mut air_left.get_main_transition_degrees());
        }
        transition_constraint_degrees.append(&mut air_right.get_main_transition_degrees());
        let context =
        // Why does Air context require at least 1 assertion?
        AirContext::new(
            trace_info, 
            transition_constraint_degrees,
            1,
            options
        );
        let width_left = <A as ComposableAir>::NUM_READ_CELLS_CURRENT;
        let width_right = <B as ComposableAir>::NUM_READ_CELLS_CURRENT;
        let num_transition_constraints_left = air_left.get_main_transition_degrees().len();
        let num_transition_constraints_right = air_right.get_main_transition_degrees().len();
        let num_periodic_values_left = air_left.get_periodic_column_values().len(); // TODO: this seems to be wasting computation of periodic cols 
        let num_periodic_values_right = air_right.get_periodic_column_values().len(); // Same here 
        MuxAir {
            context,
            air_left,
            air_right,
            width_left,
            width_right,
            num_transition_constraints_left,
            num_transition_constraints_right,
            num_periodic_values_left,
            num_periodic_values_right,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField> + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    )    {
        // Evalaute the left Air RATIO-1 times over frame chunks of size self.width_left
        evaluate_over_frame_chunks(
            &mut |frame, periodic_values, result| {
                self.air_left.evaluate_transition(frame, periodic_values, result);
            },
            &mut result[0..(RATIO - 1)*self.num_transition_constraints_left],
            frame,
            &periodic_values[0..(RATIO - 1)*self.num_periodic_values_left],
            self.width_left,
            RATIO,     
            self.num_periodic_values_left,
            self.num_transition_constraints_left        
        );

        // Evaluate the right Air, which checks constraints over the last subframe
        // of the left Air.
        let frame = EvaluationFrame::<E>::from_rows(
            frame.next()[
                (RATIO/2 - 2)*self.width_left..(RATIO/2 - 2)*self.width_left + self.width_right
            ].to_vec(),
            frame.next()[
                (RATIO/2 - 1)*self.width_left..(RATIO/2 - 1)*self.width_left + self.width_right
            ].to_vec()
        );
        let periodic_values_right = &periodic_values[
            RATIO*self.num_periodic_values_left
            ..RATIO*self.num_periodic_values_left + self.num_periodic_values_right
        ];
        self.air_right.evaluate_transition(
            &frame, 
            periodic_values_right, 
            &mut result[
                (RATIO - 1)*self.num_transition_constraints_left
                ..(RATIO - 1)*self.num_transition_constraints_left + self.num_transition_constraints_right
            ]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![Assertion::single(0, MUX_TRACE_LENGTH-1, Self::BaseField::ZERO)]
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let periodic_columns_left = split_columns(&self.air_left.get_periodic_column_values(), RATIO);
        let periodic_columns_right = self.air_right.get_periodic_column_values();
        [periodic_columns_left, periodic_columns_right].concat()
    }

}

// Helpers

pub fn split_columns<E: FieldElement>(columns: &Vec<Vec<E>>, chunk_size: usize) -> Vec<Vec<E>> {
    let mut result = vec![vec![]; columns.len()*chunk_size];
    for (col_index, column) in columns.iter().enumerate() {
        for chunk in column.chunks(chunk_size) {
            for (i, &value) in chunk.iter().enumerate() {
                result[i*columns.len() + col_index].push(value);
            }
        }
    }
    result
}

#[test]
fn test_split_columns() {
    let x = vec![
        vec![BaseElement::ZERO; 16],
        vec![BaseElement::ONE; 16],
        vec![BaseElement::ZERO; 16],
        vec![BaseElement::ONE; 16],
    ];
    let test = split_columns(&x, 4);
    let expected = vec![
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
    ];
    assert_eq!(test, expected);
}

#[test]
fn test_split_columns_small() {
    let x = vec![
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
        vec![BaseElement::ZERO; 4],
        vec![BaseElement::ONE; 4],
    ];
    let test = split_columns(&x, 2);
    let expected = vec![
        vec![BaseElement::ZERO; 2],
        vec![BaseElement::ONE; 2],
        vec![BaseElement::ZERO; 2],
        vec![BaseElement::ONE; 2],
        vec![BaseElement::ZERO; 2],
        vec![BaseElement::ONE; 2],
        vec![BaseElement::ZERO; 2],
        vec![BaseElement::ONE; 2],
    ];
    assert_eq!(test, expected);
}

fn evaluate_over_row_chunks<E, F>( 
    predicate: &mut F,
    result: &mut [E],
    row: &[E],
    raw_periodic_values: &[E],
    chunk_size: usize,
    num_periodic_values: usize,
    predicate_size: usize
) where
E: FieldElement,
F: FnMut(&EvaluationFrame<E>, &[E], &mut [E])
{
    for (((current, next), periodic_values), mut result) in 
    row.chunks(chunk_size)
    .zip(row.chunks(chunk_size).skip(1))
    .zip(raw_periodic_values.chunks(num_periodic_values))
    .zip(result.chunks_mut(predicate_size))
    {
        let frame = EvaluationFrame::<E>::from_rows(
            current.to_vec(),
            next.to_vec()
        );
        predicate(
            &frame, 
            &periodic_values, 
            &mut result
        );
    }
}

#[test]
fn test_evaluate_over_row_chunks() {
    let result = &mut [BaseElement::ZERO; 7];
    
    let mut num_calls: usize = 0;
    let mut last_next = BaseElement::ZERO;
    let mut predicate =
    |frame: &EvaluationFrame<BaseElement>, periodic_values: &[BaseElement], mut result: &mut [BaseElement]|
    {
        if num_calls > 1 {
            assert!(frame.current()[0] == last_next)
        }
        last_next = frame.next()[0];
        num_calls += 1;
        
        assert_eq!(periodic_values[0], BaseElement::ZERO);
        assert_eq!(periodic_values[1], BaseElement::ONE);
        
        result[0] = 
        (periodic_values[0] + periodic_values[1])
        *(frame.current()[0] - frame.next()[0] - BaseElement::ONE)
        *(frame.next()[0] - frame.current()[0] - BaseElement::ONE)
    };
    let row = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
    ];
    let raw_periodic_values = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
    ];
    let chunk_size = 1;
    let num_periodic_values = 2;
    let predicate_size = 1;

    evaluate_over_row_chunks(&mut predicate, result, row, raw_periodic_values, chunk_size, num_periodic_values, predicate_size);
    assert_eq!(num_calls, 7);
    assert_eq!(*result, [BaseElement::ZERO; 7]);
}

#[test]
fn test_evaluate_over_row_chunks_with_error() {
    let result = &mut [BaseElement::ZERO; 7];
    
    let mut num_calls: usize = 0;
    let mut last_next = BaseElement::ZERO;
    let mut predicate =
    |frame: &EvaluationFrame<BaseElement>, periodic_values: &[BaseElement], mut result: &mut [BaseElement]|
    {
        if num_calls > 1 {
            assert!(frame.current()[0] == last_next)
        }
        last_next = frame.next()[0];
        num_calls += 1;
        
        assert_eq!(periodic_values[0], BaseElement::ZERO);
        assert_eq!(periodic_values[1], BaseElement::ONE);
        
        result[0] = 
        (periodic_values[0] + periodic_values[1])
        *(frame.current()[0] - frame.next()[0] - BaseElement::ONE)
        *(frame.next()[0] - frame.current()[0] - BaseElement::ONE)
    };
    //Add error at step 3
    let row = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ONE, BaseElement::ZERO,BaseElement::ONE, BaseElement::ZERO,
    ];
    let raw_periodic_values = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
    ];
    let chunk_size = 1;
    let num_periodic_values = 2;
    let predicate_size = 1;

    evaluate_over_row_chunks(&mut predicate, result, row, raw_periodic_values, chunk_size, num_periodic_values, predicate_size);
    assert!(result[3] != BaseElement::ZERO);
}

fn evaluate_over_frame_chunks<E,F>(
    predicate: &mut F,
    result: &mut [E],
    frame: &EvaluationFrame<E>,
    raw_periodic_values: &[E],
    chunk_size: usize,
    num_chunks: usize,
    num_periodic_values: usize,
    predicate_size: usize,
) where
E: FieldElement,
F: FnMut(&EvaluationFrame<E>, &[E], &mut [E])
{
    // Evaluate current row
    evaluate_over_row_chunks(
        predicate,
        &mut result[0..(num_chunks/2-1)*predicate_size],
        frame.current(),
        &raw_periodic_values[0..num_chunks/2*num_periodic_values],
        chunk_size,
        num_periodic_values,
        predicate_size
    );

    // Evaluate the middle sub-frame
    let middle_frame = EvaluationFrame::<E>::from_rows(
        frame.current()[(num_chunks/2 - 1)*chunk_size..num_chunks/2*chunk_size].to_vec(),
        frame.next()[0..chunk_size].to_vec()
    );
    predicate(
        &middle_frame, 
        &raw_periodic_values[(num_chunks/2 - 1)*num_periodic_values..num_chunks/2*num_periodic_values], 
        &mut result[(num_chunks/2 - 1)*predicate_size..num_chunks/2*predicate_size]
    );

    // Evalaute the next row
    evaluate_over_row_chunks(
        predicate,
        &mut result[num_chunks/2*predicate_size..(num_chunks - 1)*predicate_size],
        frame.next(),
        &raw_periodic_values[num_chunks/2*num_periodic_values..(num_chunks - 1)*num_periodic_values],
        chunk_size,
        num_periodic_values,
        predicate_size
    );
}

#[test]
fn test_evaluate_over_frame_chunks() {
    let result = &mut [BaseElement::ZERO; 15];
    
    let mut num_calls: usize = 0;
    let mut last_next = BaseElement::ZERO;
    let mut predicate =
    |frame: &EvaluationFrame<BaseElement>, periodic_values: &[BaseElement], mut result: &mut [BaseElement]|
    {
        if num_calls > 1 {
            assert!(frame.current()[0] == last_next)
        }
        last_next = frame.next()[0];
        num_calls += 1;
        
        assert_eq!(periodic_values[0], BaseElement::ZERO);
        assert_eq!(periodic_values[1], BaseElement::ONE);
        
        // some value using the periodic values and then
        // checke that |current[0] - next[0]| = 1
        result[0] = 
        (periodic_values[0] + periodic_values[1])
        *(frame.current()[0] - frame.next()[0] - BaseElement::ONE)
        *(frame.next()[0] - frame.current()[0] - BaseElement::ONE)
    };
    let current = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
    ];
    let next = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
    ];
    let frame = EvaluationFrame::from_rows(current.to_vec(), next.to_vec());
    let raw_periodic_values = &[
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE, BaseElement::ZERO, BaseElement::ONE,
        BaseElement::ZERO, BaseElement::ONE,BaseElement::ZERO, BaseElement::ONE,
    ];
    let chunk_size = 1;
    let num_chunks = 16;
    let num_periodic_values = 2;
    let predicate_size = 1;

    evaluate_over_frame_chunks(&mut predicate, result, &frame, raw_periodic_values, chunk_size, num_chunks, num_periodic_values, predicate_size);
    assert_eq!(num_calls, 15);
    assert_eq!(*result, [BaseElement::ZERO; 15]);
}

