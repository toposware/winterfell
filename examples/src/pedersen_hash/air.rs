// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, FieldElement, ProofOptions, subsetsumair::PublicInputs};
use crate::{pedersen_hash::prover::{MUX_TRACE_WIDTH, MUX_TRACE_LENGTH}};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree
};

pub trait ComposableAir: Air {
    const NUM_READ_CELLS_CURRENT: usize;
    const NUM_READ_CELLS_NEXT: usize;
}
pub struct ZeroAir<const N: usize> {
    context: AirContext<BaseElement>
}

impl<const N: usize> ComposableAir for ZeroAir<N> {
    const NUM_READ_CELLS_CURRENT: usize = 1;
    const NUM_READ_CELLS_NEXT: usize = 0;
}

impl<const N: usize> Air for ZeroAir<N> {
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
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let mut n = N;
        n += 1;
        result[0] = frame.current()[N];
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
        let width_right = 0; // TODO: For now the second Air shares all the trace with the first Air
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
        // Evaluate the left air over the current row chunks
        evaluate_over_row_chunks(
            &mut |frame, periodic_values, result| {
                self.air_left.evaluate_transition(frame, periodic_values, result)
            },
            &mut result[0..(RATIO - 1)*self.num_transition_constraints_left],
            &frame.current(),
            &periodic_values[0..(RATIO - 1)*self.num_periodic_values_left],
            self.width_left,
            self.num_periodic_values_left,
            self.num_transition_constraints_left
        );

        // Evaluate the right Air, which checks constraints over the frame
        // of the left Air.
        let periodic_values_right = &periodic_values[
            (RATIO - 1)*self.num_periodic_values_left
            ..(RATIO - 1)*self.num_periodic_values_left + self.num_periodic_values_right
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
        vec![Assertion::single(0, MUX_TRACE_LENGTH-1, Self::BaseField::ONE)]
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

// TESTS

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

