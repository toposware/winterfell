// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{rescue, BaseElement, FieldElement, ProofOptions};
use crate::utils::{are_equal, is_zero, not, EvaluationResult};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

// CONSTANTS
// ================================================================================================

const CYCLE_LENGTH: usize = 16;

/// Specifies steps on which Rescue transition function is applied.
const CYCLE_MASK: [BaseElement; CYCLE_LENGTH] = [
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ONE,
    BaseElement::ZERO,
    BaseElement::ZERO,
];

// RESCUE AIR
// ================================================================================================

pub struct PublicInputs {
    pub seed: [BaseElement; 2],
    pub result: [BaseElement; 2],
    pub width: usize,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(&self.seed[..]);
        target.write(&self.result[..]);
    }
}

pub struct RescueAir {
    context: AirContext<BaseElement>,
    seed: [BaseElement; 2],
    result: [BaseElement; 2],
    width: usize,
}

impl Air for RescueAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let degrees =
            vec![TransitionConstraintDegree::with_cycles(3, vec![CYCLE_LENGTH]); pub_inputs.width];
        assert_eq!(pub_inputs.width, trace_info.width());
        assert!(pub_inputs.width % 4 == 0);
        RescueAir {
            context: AirContext::new(trace_info, degrees, 4, options),
            seed: pub_inputs.seed,
            result: pub_inputs.result,
            width: pub_inputs.width,
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
        // expected state width is 4 field elements
        debug_assert_eq!(self.width, current.len());
        debug_assert_eq!(self.width, next.len());

        // split periodic values into hash_flag and Rescue round constants
        let hash_flag = periodic_values[0];
        let ark = &periodic_values[1..];

        let steps = self.width / 4;
        for i in 0..steps {
            // when hash_flag = 1, constraints for Rescue round are enforced
            rescue::enforce_round(
                &mut result[i * 4..i * 4 + 4],
                &current[i * 4..i * 4 + 4],
                &next[i * 4..i * 4 + 4],
                ark,
                hash_flag,
            );

            // when hash_flag = 0, constraints for copying hash values to the next
            // step are enforced.
            let copy_flag = not(hash_flag);
            enforce_hash_copy(
                &mut result[i * 4..i * 4 + 4],
                &current[i * 4..i * 4 + 4],
                &next[i * 4..i * 4 + 4],
                copy_flag,
            );
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Assert starting and ending values of the hash chain
        let steps = self.width / 4;
        let last_step = self.trace_length() - 1;
        let mut vec = vec![
            Assertion::single(0, 0, self.seed[0]),
            Assertion::single(1, 0, self.seed[1]),
            Assertion::single(0, last_step, self.result[0]),
            Assertion::single(1, last_step, self.result[1]),
        ];

        for i in 1..steps {
            vec.append(&mut vec![
                Assertion::single(i * 4, 0, self.seed[0]),
                Assertion::single(i * 4 + 1, 0, self.seed[1]),
                Assertion::single(i * 4, last_step, self.result[0]),
                Assertion::single(i * 4 + 1, last_step, self.result[1]),
            ]);
        }

        vec
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut result = vec![CYCLE_MASK.to_vec()];
        result.append(&mut rescue::get_round_constants());
        result
    }
}

// HELPER EVALUATORS
// ------------------------------------------------------------------------------------------------

/// when flag = 1, enforces that the next state of the computation is defined like so:
/// - the first two registers are equal to the values from the previous step
/// - the other two registers are equal to 0
fn enforce_hash_copy<E: FieldElement>(result: &mut [E], current: &[E], next: &[E], flag: E) {
    result.agg_constraint(0, flag, are_equal(current[0], next[0]));
    result.agg_constraint(1, flag, are_equal(current[1], next[1]));
    result.agg_constraint(2, flag, is_zero(next[2]));
    result.agg_constraint(3, flag, is_zero(next[3]));
}
