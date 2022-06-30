// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::{are_equal, is_binary};
use winterfell::{
    Air, AirContext, Assertion, EvaluationFrame, TraceInfo, TransitionConstraintDegree,
};

// CAIRO AIR
// ================================================================================================

pub struct CairoAir {
    context: AirContext<BaseElement>,
}

impl Air for CairoAir {
    type BaseField = BaseElement;
    type PublicInputs = ();

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, public_inputs: (), options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        CairoAir {
            context: AirContext::new(trace_info, degrees, 1, options)
                .set_num_transition_exemptions(2),
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
        let one = E::from(1u128);
        let two = E::from(2u128);
        let two_to_15 = E::from(1u128 << 15);
        let two_to_16 = E::from(1u128 << 16);
        let two_to_32 = E::from(1u128 << 32);
        let two_to_48 = E::from(1u128 << 48);

        let current = frame.current();
        let next = frame.next();

        // expected state width is nb_columns field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // Flag definitions
        let f_0 = current[0] - two * current[1];
        let f_1 = current[1] - two * current[2];
        let f_2 = current[2] - two * current[3];
        let f_3 = current[3] - two * current[4];
        let f_4 = current[4] - two * current[5];
        let f_5 = current[5] - two * current[6];
        let f_6 = current[6] - two * current[7];
        let f_7 = current[7] - two * current[8];
        let f_8 = current[8] - two * current[9];
        let f_9 = current[9] - two * current[10];
        let f_10 = current[10] - two * current[11];
        let f_11 = current[11] - two * current[12];
        let f_12 = current[12] - two * current[13];
        let f_13 = current[13] - two * current[14];
        let f_14 = current[14] - two * current[15];
        let f_15 = current[15];

        let instruction_size = f_2 + one;

        // Instruction unpacking constraints
        result[0] = are_equal(
            current[20],
            current[16]
                + two_to_16 * current[17]
                + two_to_32 * current[18]
                + current[0] * two_to_48,
        ); //c_inst
           // println!("{}", current[20]);

        result[1] = is_binary(f_0);
        result[2] = is_binary(f_1);
        result[3] = is_binary(f_2);
        result[4] = is_binary(f_3);
        result[5] = is_binary(f_4);
        result[6] = is_binary(f_5);
        result[7] = is_binary(f_6);
        result[8] = is_binary(f_7);
        result[9] = is_binary(f_8);
        result[10] = is_binary(f_9);
        result[11] = is_binary(f_10);
        result[12] = is_binary(f_11);
        result[13] = is_binary(f_12);
        result[14] = is_binary(f_13);
        result[15] = is_binary(f_14);

        result[16] = current[15];

        // Operand constraints
        result[17] = are_equal(
            current[21],
            f_0 * current[28] + (one - f_0) * current[27] + (current[16] - two_to_15),
        );
        result[18] = are_equal(
            current[23],
            f_1 * current[28] + (one - f_1) * current[27] + (current[17] - two_to_15),
        );
        result[19] = are_equal(
            current[25],
            f_2 * current[19]
                + f_4 * current[27]
                + f_3 * current[28]
                + (one - f_2 - f_4 - f_3) * current[24]
                + (current[18] - two_to_15),
        );

        // ap and fp registers
        result[20] = are_equal(
            next[27],
            current[27] + f_10 * current[32] + f_11 + f_12 * two,
        );
        result[21] = are_equal(
            next[28],
            f_13 * current[22] + f_12 * (current[27] + two) + (one - f_13 - f_12) * current[28],
        );

        // pc register
        result[22] = are_equal(current[29], f_9 * current[22]);
        result[23] = are_equal(current[30], current[29] * current[32]);
        result[24] = (current[30] - f_9) * (next[19] - (current[19] + instruction_size));
        result[25] = current[29] * (next[19] - (current[19] + current[26]))
            + (one - f_9) * next[19]
            - ((one - f_7 - f_8 - f_9) * (current[19] + instruction_size)
                + f_7 * current[32]
                + f_8 * (current[19] + current[32]));

        // Opcodes and res
        result[26] = are_equal(current[31], current[24] * current[26]);
        result[27] = are_equal(
            (one - f_9) * current[32],
            f_5 * (current[24] + current[26])
                + f_6 * current[31]
                + (one - f_5 - f_6 - f_9) * current[26],
        );
        result[28] = f_12 * (current[22] - current[28]);
        result[29] = f_12 * (current[24] - (current[19] + instruction_size));
        result[30] = f_14 * (current[22] - current[32]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // DUMMY CHECK
        // Later it will be used to verify public memory.
        let last_step = self.trace_length() - 1;
        vec![Assertion::single(10, 0, Self::BaseField::ONE)]
    }
}
