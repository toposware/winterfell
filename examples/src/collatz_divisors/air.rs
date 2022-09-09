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
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
        ];

        assert_eq!(TRACE_WIDTH, trace_info.width());

        // vector to save divisors used by the air
        let mut divisors = vec![];

        // the default divisor (X^n-1)/(X-last_step)
        // period 1, offset 0, 1 final exemption
        let divisor_default = ContextDivisor::default();
        divisors.push(divisor_default);

        // We add the custom divisors

        // period 128 and offset 127 with one final ExampleOptions
        // checks transitions k*128+127 execpt the last one
        let divisor1 = ContextDivisor::new(vec![(128, 127, 1)], vec![]);
        divisors.push(divisor1);

        // all steps that are multiples of 128
        let divisor2 = ContextDivisor::new(vec![(128, 0, 0)], vec![]);
        divisors.push(divisor2);

        // check everything exept steps k*128+127
        let divisor3 = ContextDivisor::new(vec![(1, 0, 0)], vec![(128, 127)]);
        divisors.push(divisor3);

        // we assigne each constraint with one of the divisors.
        // We should use ordering with which we define the constraints
        let main_constraint_divisors: Vec<usize> = Vec::from([0, 3, 3, 3, 1, 2, 2]);

        // we mutate the divisors which by default contain only the default divisor
        CollatzAir {
            context: AirContext::new(trace_info, degrees, 2, options).set_custom_divisors(
                &divisors,
                &main_constraint_divisors,
                &[],
            ),
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
        _periodic_values: &[E],
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

        // Constraints:
        // 0. last column should contain bits (everywhere exept the last step)
        // 1,2. first column (collatz sequence) and second column (claimed remainder)
        //      remain the same except on 127 mod 128
        // 3. next bit decomposition is ok (except on 127 mod 128)
        // 4. next element in sequence is correct (on 127 mod 128)
        // 5. least significant bit correct (on 127 mod 128)
        // 6. copy of lsb is correct (on 127 mod 128)

        // last column should contain bits (everywhere)
        result[0] = current[3] * (current[3] - one);

        // first and second column remains the same (except on 127 mod 128)
        result[1] = next[0] - current[0];
        result[2] = next[1] - current[1];

        // next bit decomposition is ok (except on 127 mod 128)
        result[3] = current[2] - (next[2] * two + next[3]);

        // next element in sequence is correct (on 127 mod 128)
        // We should exclude the last step since it cycles
        result[4] = (one - current[1]) * (two * next[0] - current[0])
            + current[1] * (next[0] - (three * current[0] + one));

        // first decomposition is correct (on 0 mod 128)
        result[5] = current[0] - (current[2] * two + current[3]);

        // 6. copy of lsb is correct (on 0 mod 128)
        result[6] = current[1] - current[3];

        //
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = 128 * self.sequence_length - 1;
        vec![
            Assertion::single(0, 0, self.input_value),
            Assertion::single(0, last_step, self.final_value),
        ]
    }
}
