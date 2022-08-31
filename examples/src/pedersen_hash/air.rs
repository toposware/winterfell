// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, FieldElement, ProofOptions, prover::{TRACE_WIDTH},
    hash::{get_intial_constant_point, get_constant_points}, ecc::GENERATOR};
use crate::{utils::ecc::{enforce_point_addition_affine, AFFINE_POINT_WIDTH, POINT_COORDINATE_WIDTH}, pedersen_hash::prover::TRACE_LENGTH};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree
};

pub struct PublicInputs {
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
    }
}

// PEDERSEN HASH AIR
// ================================================================================================

pub struct PedersenHashAir<
    const TRACE_LENGTH: usize,
    const CYCLE_LENGTH: usize,
    const EXEPTIONS: usize
> {
    context: AirContext<BaseElement>
}

impl<
    const TRACE_LENGTH: usize,
    const CYCLE_LENGTH: usize,
    const EXEMPTIONS: usize
> PedersenHashAir<TRACE_LENGTH, CYCLE_LENGTH, EXEMPTIONS>
{
    const INITAL_PEDERSEN_POINT: [BaseElement; AFFINE_POINT_WIDTH] = get_intial_constant_point();
}

impl<
    const TRACE_LENGTH: usize,
    const CYCLE_LENGTH: usize,
    const EXEMPTIONS: usize
> Air for PedersenHashAir<TRACE_LENGTH, CYCLE_LENGTH, EXEMPTIONS>{
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        // This checks could be performed at compile time
        assert!(CYCLE_LENGTH.is_power_of_two() && TRACE_LENGTH.is_power_of_two());
        assert!(CYCLE_LENGTH < TRACE_LENGTH);
        assert!(EXEMPTIONS < CYCLE_LENGTH);

        let mut degrees = vec![
            TransitionConstraintDegree::with_cycles(
                2,
                vec![CYCLE_LENGTH])
        ];
        degrees.append(&mut vec![
            TransitionConstraintDegree::with_cycles(
                3,
                vec![CYCLE_LENGTH]
            );
            12
        ]);
        degrees.append(&mut vec![
            TransitionConstraintDegree::with_cycles(
                3,
                // TODO: The second cycle was only a guess and I still nedd to understand how is this happening
                vec![TRACE_LENGTH, CYCLE_LENGTH]
            );
            6
        ]);

        assert_eq!(TRACE_WIDTH, trace_info.layout().virtual_trace_width());
        let context =
            // Why does Air context require at least 1 assertion?
            AirContext::new(
                trace_info, 
                degrees,
                12,
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
        let bit_unpacking_flag = E::ONE - subset_sum_flag;

        // constraint pedersen/hash0/ec_subset_sum/booleanity_test 
        result[0] = subset_sum_flag*bit*(bit - E::ONE);
        // result[1..POINT_COORDINATE_WIDTH+1] corresponds to constraint pedersen/hash0/ec_subset_sum/add_points/slope
        // result[POINT_COORDINATE_WIDTH + 1 .. 2*POINT_COORDINATE_WIDTH + 1] corresponds to pedersen/hash0/ec_subset_sum/add_points/x
        // result[2*POINT_COORDINATE_WIDTH + 1 .. 3*POINT_COORDINATE_WIDTH + 1] corresponds to pedersen/hash0/ec_subset_sum/add_points/y
        // TODO: constraints pedersen/hash0/ec_subset_sum/copy_point/x and pedersen/hash0/ec_subset_sum/copy_point/y are missing. 
        enforce_point_addition_affine(
            &mut result[1..],
            lhs,
            rhs,
            slope,
            point,
            bit*subset_sum_flag);

        // Constraints pedersen/hash0/ec_subset_sum/bit_unpacking/ require virtual columns

    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let mut initial_point_assertions = Vec::new();
        for i in 0..AFFINE_POINT_WIDTH {
            initial_point_assertions.push(Assertion::single(i, 0, Self::INITAL_PEDERSEN_POINT[i]));
        }
        initial_point_assertions
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut periodic_columns = vec![Vec::new(); AFFINE_POINT_WIDTH];
        for point in get_constant_points::<TRACE_LENGTH>().into_iter() {
            for (i, column) in periodic_columns.iter_mut().enumerate() {
                column.push(point[i]);
            }
        }
        let mut subset_sum_flags = vec![BaseElement::ONE; CYCLE_LENGTH];
        subset_sum_flags[CYCLE_LENGTH - 1] = BaseElement::ZERO;
        periodic_columns.push(subset_sum_flags);
        periodic_columns
    }
}
