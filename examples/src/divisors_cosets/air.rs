// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

// DIVISORS COSETS AIR
// ================================================================================================

// Public inputs contain the exponentiation result and the second divisor info (length, offset)
#[derive(Clone)]
pub struct PublicInputs {
    pub result: BaseElement,
    pub range_length: u64,
    pub offset: u64,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.result);
        target.write_u64(self.range_length);
        target.write_u64(self.offset);
    }
}

pub struct DivisorsCosetsAir {
    context: AirContext<BaseElement>,
    public_inputs: PublicInputs,
}

impl Air for DivisorsCosetsAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(1),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());

        // initially we only include the default divisor
        // period = 1 with offset = 0 and 1 point exemptions, no complex exemptions
        let default_divisor = (vec![(1, 0, 1)], vec![]);

        // custom divisor for range checks: period = trace_length/range_length offset given by pub_inputs
        // and no exclustion point. No complex exemptions
        let custom_divisor: (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) = (
            vec![(
                trace_info.length() / pub_inputs.range_length as usize,
                pub_inputs.offset as usize,
                1,
            )],
            vec![],
        );

        let divisors = vec![default_divisor, custom_divisor];

        // We add the custom divisor
        let custom_divisor = vec![(
            pub_inputs.range_length as usize,
            pub_inputs.offset as usize,
            0,
        )];

        // first constarint uses the defualt divisor
        // the other two constraints use the custom divisor
        let main_constraint_divisors: Vec<usize> = vec![0, 1, 1];
        let aux_constraint_divisors: Vec<usize> = vec![];

        DivisorsCosetsAir {
            context: AirContext::new(trace_info, degrees, 2, options).set_custom_divisors(
                &divisors,
                &main_constraint_divisors,
                &aux_constraint_divisors,
            ),
            public_inputs: pub_inputs,
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
        let current = frame.current();
        let next = frame.next();
        // expected state width is 2 field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // TODO: [divisors] create different divisors for examples

        result[0] = are_equal(next[0], current[0] + current[0]);
        result[1] = (current[1] - E::ONE) * current[1];
        result[2] = next[1] - (E::ONE - current[1]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, Self::BaseField::ONE),
            Assertion::single(0, last_step, self.public_inputs.result),
        ]
    }
}
