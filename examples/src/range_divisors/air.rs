// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, ByteWriter, ContextDivisor, EvaluationFrame, ProofOptions,
    Serializable, TraceInfo, TransitionConstraintDegree,
};

// COLLATZ AIR
// ================================================================================================

pub(crate) const TRACE_WIDTH: usize = 3;
pub(crate) const PERIOD: usize = 8;

pub struct PublicInputs {
    pub input_value: BaseElement,
    pub sequence_length: usize,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.input_value);
        target.write(BaseElement::from(self.sequence_length as u64));
    }
}

pub struct RangeAir {
    context: AirContext<BaseElement>,
}

impl Air for RangeAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let mut degrees = vec![];
        // 64 constraints in total
        for _ in 0..PERIOD {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(1));
        degrees.push(TransitionConstraintDegree::new(2));

        assert_eq!(TRACE_WIDTH, trace_info.width());

        // vector to save divisors used by the air
        let mut divisors = vec![];

        // the default divisor (X^n-1)/(X-last_step)
        // period 1, offset 0, 1 final exemption
        // let divisor_default = (vec![(1, 0, 1)], vec![]);
        // divisors.push(divisor_default);

        // We add the custom divisors

        // divisors for first column
        for i in 0..PERIOD {
            // period PERIOD, offset i, no exemptions
            let divisor = ContextDivisor::new(vec![(PERIOD, i, 0)], vec![]);
            divisors.push(divisor);
        }

        // divisors for second column
        let divisor2 = ContextDivisor::new(vec![(512, 512 - 42, 0)], vec![]);
        let divisor2complement = ContextDivisor::new(vec![(1, 0, 0)], vec![(512, 512 - 42)]);
        divisors.push(divisor2);
        divisors.push(divisor2complement);

        // divisors for third column
        let divisor3 = ContextDivisor::new(vec![(1024, 41, 0)], vec![]);
        let divisor3complement = ContextDivisor::new(vec![(1, 0, 1)], vec![(1024, 41)]);
        divisors.push(divisor3);
        divisors.push(divisor3complement);

        // we assigne each constraint with one of the divisors.
        // We should use ordering with which we define the constraints
        let mut main_constraint_divisors: Vec<usize> = vec![];
        for i in 0..PERIOD {
            main_constraint_divisors.push(i);
        }
        main_constraint_divisors.push(PERIOD + 1);
        main_constraint_divisors.push(PERIOD);
        main_constraint_divisors.push(PERIOD + 3);
        main_constraint_divisors.push(PERIOD + 2);

        // we mutate the divisors which by default contain only the default divisor
        RangeAir {
            context: AirContext::new(trace_info, degrees, 3, options).set_custom_divisors(
                &divisors,
                &main_constraint_divisors,
                &[],
            ),
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
        // expected state width is TRACE_WIDTH field elements large
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // checks for column 1
        for i in 0..PERIOD {
            let shift = current[0] - E::from(i as u128);
            result[i] = shift * (shift - E::ONE);
        }
        // checks for column 2
        // 1. bit checks
        result[PERIOD] = current[1] * (E::ONE - current[1]);
        // 2. 42/43 check
        result[PERIOD + 1] = (current[1] - E::from(42u128)) * (current[1] - E::from(43u128));

        // checks for column 3
        result[PERIOD + 2] = next[2] - (current[0] + next[0]);
        result[PERIOD + 3] = next[2] - (current[0] * next[0]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![
            Assertion::single(0, 0, BaseElement::ZERO),
            Assertion::single(1, 0, BaseElement::ZERO),
            Assertion::single(2, 0, BaseElement::ONE),
        ]
    }
}
