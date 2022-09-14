// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, ProofOptions, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

// COLLATZ AIR
// ================================================================================================

pub(crate) const TRACE_WIDTH: usize = 3;
pub(crate) const PERIOD: usize = 128;

pub struct PublicInputs {
    pub input_value: BaseElement,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.input_value);
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
            degrees.push(TransitionConstraintDegree::with_complex_cycles(
                2,
                vec![(PERIOD, 1)],
            ));
        }
        degrees.push(TransitionConstraintDegree::with_complex_cycles(
            2,
            vec![(PERIOD / 2, PERIOD / 2 - 1)],
        ));
        degrees.push(TransitionConstraintDegree::with_complex_cycles(
            2,
            vec![(PERIOD / 2, 1)],
        ));
        degrees.push(TransitionConstraintDegree::with_complex_cycles(
            1,
            vec![(PERIOD * 2, PERIOD * 2 - 1)],
        ));
        degrees.push(TransitionConstraintDegree::with_complex_cycles(
            2,
            vec![(PERIOD * 2, 1)],
        ));

        assert_eq!(TRACE_WIDTH, trace_info.width());

        // we mutate the divisors which by default contain only the default divisor
        RangeAir {
            context: AirContext::new(trace_info, degrees, 3, options),
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
        // expected state width is TRACE_WIDTH field elements large
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // checks for column 0
        for i in 0..PERIOD {
            let shift = current[0] - E::from(i as u128);
            result[i] = periodic_values[i] * (shift * (shift - E::ONE));
        }
        // checks for column 1
        // 1. bit checks
        result[PERIOD] = periodic_values[PERIOD] * (next[1] * (E::ONE - next[1]));
        // 2. 42/43 check
        result[PERIOD + 1] =
            periodic_values[PERIOD + 1] * (next[1] - E::from(42u128)) * (next[1] - E::from(43u128));

        // checks for column 2
        result[PERIOD + 2] = periodic_values[PERIOD + 2] * (next[2] - (current[0] + next[0]));
        result[PERIOD + 3] = periodic_values[PERIOD + 3] * (next[2] - (current[0] * next[0]));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![
            Assertion::single(0, 0, BaseElement::ZERO),
            Assertion::single(1, 0, BaseElement::ZERO),
            Assertion::single(2, 0, BaseElement::ONE),
        ]
    }

    fn get_custom_divisors(&self) -> Vec<(usize, Vec<usize>)> {
        // column 0 constraints
        let mut pv = vec![];
        for i in 0..PERIOD {
            pv.push((PERIOD, vec![i]));
        }
        // column 1 constraints
        let mut offsets = vec![];
        for i in 0..PERIOD / 2 {
            if i != PERIOD / 2 - 42 - 1 {
                offsets.push(i);
            }
        }
        pv.push((PERIOD / 2, offsets));
        pv.push((PERIOD / 2, vec![PERIOD / 2 - 42 - 1]));
        // column 2 constraints
        let mut offsets = vec![];
        for i in 0..2 * PERIOD {
            if i != 41 {
                offsets.push(i);
            }
        }
        pv.push((PERIOD * 2, offsets));
        pv.push((PERIOD * 2, vec![41]));

        pv
    }
}
