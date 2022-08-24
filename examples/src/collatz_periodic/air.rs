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

pub(crate) const TRACE_WIDTH: usize = 4;

pub struct PublicInputs {
    pub input_value: BaseElement,
    pub final_value: BaseElement,
    pub sequence_length: usize,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.input_value);
        target.write(self.final_value);
        target.write(BaseElement::from(self.sequence_length as u64));
    }
}

pub struct CollatzAir {
    context: AirContext<BaseElement>,
    input_value: BaseElement,
    final_value: BaseElement,
    sequence_length: usize,
}

impl Air for CollatzAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::with_cycles(1, vec![128]),
            TransitionConstraintDegree::with_cycles(1, vec![128]),
            TransitionConstraintDegree::with_cycles(1, vec![128]),
            TransitionConstraintDegree::with_cycles(2, vec![128]),
            TransitionConstraintDegree::with_cycles(1, vec![128]),
            TransitionConstraintDegree::with_cycles(1, vec![128]),
        ];

        assert_eq!(TRACE_WIDTH, trace_info.width());
        CollatzAir {
            context: AirContext::new(trace_info, degrees, 2, options),
            input_value: pub_inputs.input_value,
            final_value: pub_inputs.final_value,
            sequence_length: pub_inputs.sequence_length,
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

        let _zero = E::ZERO;
        let one = E::ONE;
        let two = E::from(2u128);
        let three = E::from(3u128);

        // check conditions on 127 mod 128 or not
        let flag = periodic_values[0];

        // Constraints:
        // 1. last column should contain bits (everywhere)
        // 2. first and second column remains the same (except on 127 mod 128)
        // 3. next bit decomposition is ok (except on 127 mod 128)
        // 4. next element in sequence is correct (on 127 mod 128)
        // 5. least significant bit correct (on 127 mod 128)
        // 6. copy of lsb is correct (on 127 mod 128)

        // last column should contain bits (everywhere)
        result[0] = current[3] * (current[3] - one);

        // first and second column remains the same (except on 127 mod 128)
        result[1] = (one - flag) * (next[0] - current[0]);
        result[2] = (one - flag) * (next[1] - current[1]);

        // next bit decomposition is ok (except on 127 mod 128)
        result[3] = (one - flag) * (current[2] - (next[2] * two + next[3]));

        // next element in sequence is correct (on 127 mod 128)
        result[4] = flag
            * ((one - current[1]) * (two * next[0] - current[0])
                + current[1] * (next[0] - (three * current[0] + one)));

        // first decomposition is correct (on 127 mod 128)
        result[5] = flag * (next[0] - (next[2] * two + next[3]));

        // 6. copy of lsb is correct (on 127 mod 128)
        result[6] = flag * (next[1] - next[3]);

        //
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = 128 * self.sequence_length - 1;
        vec![
            Assertion::single(0, 0, self.input_value),
            Assertion::single(0, last_step, self.final_value),
        ]
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut pv = Vec::new();
        let period = (0..128)
            .map(|x| {
                if x == 127 {
                    BaseElement::ONE
                } else {
                    BaseElement::ZERO
                }
            })
            .collect::<Vec<_>>();
        pv.push(period);
        pv
    }
}
