// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, FieldElement, ProofOptions, prover::{TRACE_WIDTH, BITS_PER_CHUNK},
    hash::get_constant_points, ecc::GENERATOR};
use crate::utils::ecc::{enforce_point_addition_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree,
};

pub struct PublicInputs {
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
    }
}

// PEDERSEN HASH AIR
// ================================================================================================

pub struct PedersenHashAir<const CYCLE_LENGTH: usize, const EXEPTIONS: usize> {
    context: AirContext<BaseElement>
}

impl<const CYCLE_LENGTH: usize, const EXEMPTIONS: usize> Air for PedersenHashAir<CYCLE_LENGTH, EXEMPTIONS>{
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let mut degrees = vec![
            TransitionConstraintDegree::with_cycles(
                2,
                vec![BITS_PER_CHUNK.next_power_of_two()])
        ];
        degrees.append(&mut vec![
            TransitionConstraintDegree::with_cycles(
                2,
                vec![CYCLE_LENGTH, BITS_PER_CHUNK.next_power_of_two()]
            );
            12
        ]);
        degrees.append(&mut vec![
            TransitionConstraintDegree::with_cycles(
                2,
                vec![BITS_PER_CHUNK.next_power_of_two(), BITS_PER_CHUNK.next_power_of_two()]
            );
            6
        ]);

        assert_eq!(TRACE_WIDTH, trace_info.layout().virtual_trace_width());
        let context =
            // Why does Air context require at least 1 assertion?
            AirContext::new(
                trace_info, 
                degrees,
                1,
                options).set_num_transition_exemptions(EXEMPTIONS);
        PedersenHashAir {
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

        let lhs = &current[..AFFINE_POINT_WIDTH];
        let rhs = &periodic_values[..AFFINE_POINT_WIDTH];
        let slope = &current[AFFINE_POINT_WIDTH + 1 .. AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1];
        let point = &next[..AFFINE_POINT_WIDTH];
        let bit = current[AFFINE_POINT_WIDTH] - (E::ONE + E::ONE)*next[AFFINE_POINT_WIDTH];
        let subset_sum_flag = periodic_values[AFFINE_POINT_WIDTH];

        result[0] = subset_sum_flag*bit*(bit - E::ONE);
        enforce_point_addition_affine(
            &mut result[1..],
            lhs,
            rhs,
            slope,
            point,
            bit*subset_sum_flag);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Add a dummy assetion for now.
        vec![
            Assertion::single(0, 0, GENERATOR[0])
        ]
    }
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut periodic_columns = vec![Vec::new(); AFFINE_POINT_WIDTH];
        for point in get_constant_points::<CYCLE_LENGTH>().into_iter() {
            for (i, column) in periodic_columns.iter_mut().enumerate() {
                column.push(point[i]);
            }
        }
        let mut subset_sum_flags = vec![BaseElement::ONE; BITS_PER_CHUNK.next_power_of_two()];
        subset_sum_flags[15] = BaseElement::ZERO;
        periodic_columns.push(subset_sum_flags);
        periodic_columns
    }
}
