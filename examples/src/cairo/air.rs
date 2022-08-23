// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, ExtensionOf, FieldElement, ProofOptions, AUX_WIDTH, TRACE_WIDTH};
use crate::utils::{are_equal, is_binary};
use winterfell::{
    Air, AirContext, Assertion, AuxTraceRandElements, ByteWriter, EvaluationFrame, Serializable,
    TraceInfo, TransitionConstraintDegree,
};

// CAIRO AIR
// ================================================================================================

pub struct PublicInputs {
    pub bytecode: Vec<BaseElement>,
    // pc_0, pc_final, ap_0, ap_final
    pub register_values: Vec<BaseElement>,
    // output_begin, output_stop
    pub output_pointer_values: Vec<BaseElement>,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(&self.bytecode[..]);
        target.write(&self.register_values[..]);
        target.write(&self.output_pointer_values[..]);
    }
}

pub struct CairoAir {
    context: AirContext<BaseElement>,
    public_inputs: PublicInputs,
}

impl Air for CairoAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, public_inputs: PublicInputs, options: ProofOptions) -> Self {
        let main_degrees = vec![
            // CPU constraints
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
            // Offset range checks
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            // Memory accesses contiguity and read-only
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
        let aux_degrees = vec![
            // Offset permutation constraints
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            // Memory permutation constraints
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
        ];
        assert_eq!(TRACE_WIDTH + AUX_WIDTH, trace_info.width());
        CairoAir {
            context: AirContext::new_multi_segment(
                trace_info,
                main_degrees,
                aux_degrees,
                7,
                2,
                options,
            )
            .set_num_transition_exemptions(2),
            public_inputs: public_inputs,
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
        let one = E::ONE;
        let two = one.double();
        let two_to_15 = E::from(1u128 << 15);
        let two_to_16 = two_to_15.double();
        let two_to_32 = two_to_16.square();
        let two_to_48 = two_to_32 * two_to_16;

        let current = frame.current();
        let next = frame.next();

        // expected state width is nb_columns field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // Flag definitions
        let f_0 = current[0] - current[1].double();
        let f_1 = current[1] - current[2].double();
        let f_2 = current[2] - current[3].double();
        let f_3 = current[3] - current[4].double();
        let f_4 = current[4] - current[5].double();
        let f_5 = current[5] - current[6].double();
        let f_6 = current[6] - current[7].double();
        let f_7 = current[7] - current[8].double();
        let f_8 = current[8] - current[9].double();
        let f_9 = current[9] - current[10].double();
        let f_10 = current[10] - current[11].double();
        let f_11 = current[11] - current[12].double();
        let f_12 = current[12] - current[13].double();
        let f_13 = current[13] - current[14].double();
        let f_14 = current[14] - current[15].double();

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
            current[27] + f_10 * current[32] + f_11 + f_12.double(),
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

        // Offset range checks
        result[31] = (current[35] - current[34]) * (current[35] - current[34] - one);
        result[32] = (current[36] - current[35]) * (current[36] - current[35] - one);
        result[33] = (current[37] - current[36]) * (current[37] - current[36] - one);
        result[34] = (next[34] - current[37]) * (next[34] - current[37] - one);

        // Memory accesses contiguity and read-only
        result[35] = (current[42] - current[40]) * (current[42] - current[40] - one);
        result[36] = (current[44] - current[42]) * (current[44] - current[42] - one);
        result[37] = (current[46] - current[44]) * (current[46] - current[44] - one);
        result[38] = (current[48] - current[46]) * (current[48] - current[46] - one);
        result[39] = (next[40] - current[48]) * (next[40] - current[48] - one);

        result[40] = (current[43] - current[41]) * (current[42] - current[40] - one);
        result[41] = (current[45] - current[43]) * (current[44] - current[42] - one);
        result[42] = (current[47] - current[45]) * (current[46] - current[44] - one);
        result[43] = (current[49] - current[47]) * (current[48] - current[46] - one);
        result[44] = (next[41] - current[49]) * (next[40] - current[48] - one);
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        periodic_values: &[F],
        aux_rand_elements: &AuxTraceRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        let main_current = main_frame.current();
        let main_next = main_frame.next();

        let aux_current = aux_frame.current();
        let aux_next = aux_frame.next();

        let random_elements = aux_rand_elements.get_segment_elements(0);

        // We want to enforce that the sorted columns are a permutation of the
        // original main segment columns. Instead of a classical permutation
        // check between two columns, we check it in a "snake way":
        // If C_0 to C_3 are original columns and S_0 to S_3 are sorted, then
        // we check that C_00, C_10, C_20, C_30, C_01, ... is a permutation of
        // S_00, S_10, S_20, S_30, S_01, ...

        // Offset permutation arguments
        result[0] = aux_current[1] * (random_elements[0] - main_current[35].into())
            - aux_current[0] * (random_elements[0] - main_current[17].into());
        result[1] = aux_current[2] * (random_elements[0] - main_current[36].into())
            - aux_current[1] * (random_elements[0] - main_current[18].into());
        result[2] = aux_current[3] * (random_elements[0] - main_current[37].into())
            - aux_current[2] * (random_elements[0] - main_current[33].into());
        result[3] = aux_next[0] * (random_elements[0] - main_next[34].into())
            - aux_current[3] * (random_elements[0] - main_next[16].into());

        // Memory permutation arguments
        // Necessary variables: into() fails otherwise. Any better way to do this?
        let mut a: E = main_current[21].into();
        let mut a2: E = main_current[42].into();
        result[4] = aux_current[5]
            * (random_elements[1] - (a2 + random_elements[2] * main_current[43].into()))
            - aux_current[4]
                * (random_elements[1] - (a + random_elements[2] * main_current[22].into()));
        a = main_current[23].into();
        a2 = main_current[44].into();
        result[5] = aux_current[6]
            * (random_elements[1] - (a2 + random_elements[2] * main_current[45].into()))
            - aux_current[5]
                * (random_elements[1] - (a + random_elements[2] * main_current[24].into()));
        a = main_current[25].into();
        a2 = main_current[46].into();
        result[6] = aux_current[7]
            * (random_elements[1] - (a2 + random_elements[2] * main_current[47].into()))
            - aux_current[6]
                * (random_elements[1] - (a + random_elements[2] * main_current[26].into()));
        a = main_current[38].into();
        a2 = main_current[48].into();
        result[7] = aux_current[8]
            * (random_elements[1] - (a2 + random_elements[2] * main_current[49].into()))
            - aux_current[7]
                * (random_elements[1] - (a + random_elements[2] * main_current[39].into()));
        a = main_current[50].into();
        a2 = main_current[52].into();
        result[8] = aux_current[9]
            * (random_elements[1] - (a2 + random_elements[2] * main_current[53].into()))
            - aux_current[7]
                * (random_elements[1] - (a + random_elements[2] * main_current[51].into()));
        a = main_next[19].into();
        a2 = main_next[40].into();
        result[8] = aux_next[4]
            * (random_elements[1] - (a2 + random_elements[2] * main_next[41].into()))
            - aux_current[9]
                * (random_elements[1] - (a + random_elements[2] * main_next[20].into()));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Check boundary register and built-in pointer values.
        let last_step = self.trace_length() - 2;
        vec![
            Assertion::single(19, 0, self.public_inputs.register_values[0]),
            Assertion::single(19, last_step, self.public_inputs.register_values[1]),
            Assertion::single(27, 0, self.public_inputs.register_values[2]),
            Assertion::single(28, 0, self.public_inputs.register_values[2]),
            Assertion::single(27, last_step, self.public_inputs.register_values[3]),
            Assertion::single(50, 0, self.public_inputs.output_pointer_values[0]),
            Assertion::single(
                50,
                last_step,
                self.public_inputs.output_pointer_values[1] - Self::BaseField::ONE,
            ),
        ]
    }

    fn get_aux_assertions<E: FieldElement + From<Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let mut final_value = E::ONE;
        let mut a: E;
        let bytecode = self.public_inputs.bytecode.clone();
        // println!("{:#?}", bytecode);
        let random_elements = aux_rand_elements.get_segment_elements(0);

        for i in 0..(bytecode.len() / 2) {
            a = bytecode[2 * i].into();
            final_value *= random_elements[1]
                / (random_elements[1] - (a + random_elements[2] * bytecode[2 * i + 1].into()));
        }

        vec![
            Assertion::single(3, self.trace_length() - 2, E::ONE),
            Assertion::single(9, self.trace_length() - 2, final_value),
        ]
    }
}
