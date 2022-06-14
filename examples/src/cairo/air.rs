// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, EvaluationFrame, TraceInfo, TransitionConstraintDegree,
};

// CAIRO AIR
// ================================================================================================

pub struct CairoAir {
    context: AirContext<BaseElement>,
    result: BaseElement,
}

impl Air for CairoAir {
    type BaseField = BaseElement;
    type PublicInputs = BaseElement;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::BaseField, options: ProofOptions) -> Self {
        let degrees = vec![
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
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        CairoAir {
            context: AirContext::new(trace_info, degrees, 1, options),
            result: pub_inputs,
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
        let f_0 = current[0] - two*current[1];
        let f_1 = current[1] - two*current[2];
        let f_2 = current[2] - two*current[3];
        let f_3 = current[3] - two*current[4];
        let f_4 = current[4] - two*current[5];
        let f_5 = current[5] - two*current[6];
        let f_6 = current[6] - two*current[7];
        let f_7 = current[7] - two*current[8];
        let f_8 = current[8] - two*current[9];
        let f_9 = current[9] - two*current[10];
        let f_10 = current[10] - two*current[11];
        let f_11 = current[11] - two*current[12];
        let f_12 = current[12] - two*current[13];
        let f_13 = current[13] - two*current[14];
        let f_14 = current[14] - two*current[15];
        let f_15 = current[15];

        let instruction_size = f_2 + one;

        // Instruction unpacking constraints
        result[0] = are_equal(current[20], current[16] + two_to_16*current[17] + two_to_32*current[18] + current[0]*two_to_48); //c_inst
        // println!("{}", current[20]);

        result[1] = f_0 * (f_0 - one);
        result[2] = f_1 * (f_1 - one);
        result[3] = f_2 * (f_2 - one);
        result[4] = f_3 * (f_3 - one);
        result[5] = f_4 * (f_4 - one);
        result[6] = f_5 * (f_5 - one);
        result[7] = f_6 * (f_6 - one);
        result[8] = f_7 * (f_7 - one);
        result[9] = f_8 * (f_8 - one);
        result[10] = f_9 * (f_9 - one);
        result[11] = f_10 * (f_10 - one);
        result[12] = f_11 * (f_11 - one);
        result[13] = f_12 * (f_12 - one);
        result[14] = f_13 * (f_13 - one);
        result[15] = f_14 * (f_14 - one);
        result[16] = f_15 * (f_15 - one);

        result[17] = current[15];

        // Operand constraints
        result[18] = are_equal(current[21], f_0*current[28] + (one - f_0)*current[27] + (current[16] - two_to_15));
        result[19] = are_equal(current[23], f_1*current[28] + (one - f_1)*current[27] + (current[17] - two_to_15));
        result[20] = are_equal(current[25], f_2*current[19] + f_4*current[27] + f_3*current[28] + (one - f_2 - f_4 - f_3)*current[24] + (current[18] - two_to_15));

        // ap and fp registers
        result[21] = are_equal(next[27], current[27] + f_10*current[32] + f_11 + f_12*two);
        result[22] = are_equal(next[28], f_13*current[22] + f_12*(current[27] + two) + (one - f_13 - f_12)*current[28]);

        // pc register
        result[23] = are_equal(current[29], f_9*current[22]);
        result[24] = are_equal(current[30], current[29]*current[32]);
        result[25] = (current[30] - f_9)*(next[19] - (current[19] + instruction_size));
        result[26] = current[29]*(current[19] - (next[19] + current[26])) + (one - f_9)*next[19] - ((one - f_7 - f_8 - f_9)*(current[19] + instruction_size) + f_7*current[32] + f_8*(current[19] + current[32]));

        // Opcodes and res
        result[27] = are_equal(current[31], current[24]*current[26]);
        result[28] = are_equal((one - f_9)*current[32], f_5*(current[24] + current[26]) + f_6*current[31] + (one - f_5 - f_6 - f_9)*current[26]);
        result[29] = f_12*(current[22] - current[28]);
        result[30] = f_12*(current[24] - (current[19] + instruction_size));
        result[31] = f_14*(current[22] - current[32]);

    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // DUMMY CHECK
        // Later it will be used to verify public memory.
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(10, 0, Self::BaseField::ONE),
        ]
    }
}
