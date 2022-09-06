// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, ExtensionOf, FieldElement, ProofOptions, ALPHA, ARK, AUX_WIDTH, INV_ALPHA,
    INV_MDS, MDS, MEMORY_COLUMNS, NB_MEMORY_COLUMN_PAIRS, NB_OFFSET_COLUMNS, NUM_ROUNDS,
    OFFSET_COLUMNS, SORTED_MEMORY_COLUMNS, SORTED_OFFSET_COLUMNS, STATE_WIDTH, TRACE_WIDTH,
};

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
    // rescue_begin, rescue_stop
    pub rescue_pointer_values: Vec<BaseElement>,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(&self.bytecode[..]);
        target.write(&self.register_values[..]);
        target.write(&self.rescue_pointer_values[..]);
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
        let mut main_degrees = vec![
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
        ];

        // Contiguity constraints
        for _ in 0..NB_OFFSET_COLUMNS {
            main_degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..NB_MEMORY_COLUMN_PAIRS {
            main_degrees.push(TransitionConstraintDegree::new(2));
        }

        // Read-only constraints
        for _ in 0..NB_MEMORY_COLUMN_PAIRS {
            main_degrees.push(TransitionConstraintDegree::new(2));
        }

        // Rescue constraints
        for _ in 0..(STATE_WIDTH * NUM_ROUNDS) {
            main_degrees.push(TransitionConstraintDegree::new(7));
        }

        let mut aux_degrees = vec![];

        // Offset permutation constraints
        for _ in 0..NB_OFFSET_COLUMNS {
            aux_degrees.push(TransitionConstraintDegree::new(2));
        }
        // Memory permuation constraints
        for _ in 0..NB_MEMORY_COLUMN_PAIRS {
            aux_degrees.push(TransitionConstraintDegree::new(2));
        }

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

        // Sorted column constraints
        let mut constraints_offset = 31;
        enforce_contiguity_constraints(result, constraints_offset, current, next);
        constraints_offset += NB_OFFSET_COLUMNS + NB_MEMORY_COLUMN_PAIRS;
        enforce_read_only_constraints(result, constraints_offset, current, next);

        // Rescue constraints
        constraints_offset += NB_MEMORY_COLUMN_PAIRS;
        let mut current_state = [E::ZERO; STATE_WIDTH];
        let mut next_state = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            current_state[i] = current[51 + 2 * i];
            next_state[i] = current[98 + i];
            println!("{}\n", next_state[i]);
        }
        for round in 0..(NUM_ROUNDS - 1) {
            enforce_round(
                result,
                constraints_offset,
                &current_state,
                &next_state,
                round,
            );

            current_state = next_state;
            for i in 0..STATE_WIDTH {
                next_state[i] = current[98 + (round + 1) * STATE_WIDTH + i];
            }
            constraints_offset += STATE_WIDTH;
        }
        for i in 0..STATE_WIDTH {
            next_state[i] = current[75 + 2 * i];
            println!("{}\n", next_state[i]);
        }
        enforce_round(
            result,
            constraints_offset,
            &current_state,
            &next_state,
            NUM_ROUNDS - 1,
        );
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
        enforce_offset_aux_constraints(
            result,
            0,
            0,
            main_current,
            main_next,
            aux_current,
            aux_next,
            random_elements[0],
        );

        enforce_memory_aux_constraints(
            result,
            4,
            4,
            main_current,
            main_next,
            aux_current,
            aux_next,
            random_elements[1],
            random_elements[2],
        )
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Check boundary register values.
        let last_step = self.trace_length() - 2;
        vec![
            Assertion::single(19, 0, self.public_inputs.register_values[0]),
            Assertion::single(19, last_step, self.public_inputs.register_values[1]),
            Assertion::single(27, 0, self.public_inputs.register_values[2]),
            Assertion::single(28, 0, self.public_inputs.register_values[2]),
            Assertion::single(27, last_step, self.public_inputs.register_values[3]),
            Assertion::single(50, 0, self.public_inputs.rescue_pointer_values[0]),
            Assertion::single(
                96,
                last_step,
                self.public_inputs.rescue_pointer_values[1] - Self::BaseField::ONE,
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
            Assertion::single(NB_OFFSET_COLUMNS - 1, self.trace_length() - 2, E::ONE),
            Assertion::single(AUX_WIDTH - 1, self.trace_length() - 2, final_value),
        ]
    }
}

// HELPER EVALUATORS
// ------------------------------------------------------------------------------------------------

/// Enforces contiguity constraints in sorted columns.
fn enforce_contiguity_constraints<E: FieldElement>(
    result: &mut [E],
    result_offset: usize,
    current: &[E],
    next: &[E],
) {
    let one = E::ONE;
    // Contiguity in sorted offset columns.
    for i in 0..(NB_OFFSET_COLUMNS - 1) {
        result[result_offset + i] = (current[SORTED_OFFSET_COLUMNS[i + 1]]
            - current[SORTED_OFFSET_COLUMNS[i]])
            * (current[SORTED_OFFSET_COLUMNS[i + 1]] - current[SORTED_OFFSET_COLUMNS[i]] - one);
    }
    result[result_offset + (NB_OFFSET_COLUMNS - 1)] = (next[SORTED_OFFSET_COLUMNS[0]]
        - current[SORTED_OFFSET_COLUMNS[NB_OFFSET_COLUMNS - 1]])
        * (next[SORTED_OFFSET_COLUMNS[0]]
            - current[SORTED_OFFSET_COLUMNS[NB_OFFSET_COLUMNS - 1]]
            - one);
    // Contiguity in sorted memory addresses.
    for i in 0..(NB_MEMORY_COLUMN_PAIRS - 1) {
        result[result_offset + NB_OFFSET_COLUMNS + i] = (current[SORTED_MEMORY_COLUMNS[i + 1].0]
            - current[SORTED_MEMORY_COLUMNS[i].0])
            * (current[SORTED_MEMORY_COLUMNS[i + 1].0] - current[SORTED_MEMORY_COLUMNS[i].0] - one);
    }
    result[result_offset + NB_OFFSET_COLUMNS + (NB_MEMORY_COLUMN_PAIRS - 1)] = (next
        [SORTED_MEMORY_COLUMNS[0].0]
        - current[SORTED_MEMORY_COLUMNS[NB_MEMORY_COLUMN_PAIRS - 1].0])
        * (next[SORTED_MEMORY_COLUMNS[0].0]
            - current[SORTED_MEMORY_COLUMNS[NB_MEMORY_COLUMN_PAIRS - 1].0]
            - one);
}

/// Enforces read_only constraints in sorted memory columns.
fn enforce_read_only_constraints<E: FieldElement>(
    result: &mut [E],
    result_offset: usize,
    current: &[E],
    next: &[E],
) {
    let one = E::ONE;
    for i in 0..(NB_MEMORY_COLUMN_PAIRS - 1) {
        result[result_offset + i] = (current[SORTED_MEMORY_COLUMNS[i + 1].1]
            - current[SORTED_MEMORY_COLUMNS[i].1])
            * (current[SORTED_MEMORY_COLUMNS[i + 1].0] - current[SORTED_MEMORY_COLUMNS[i].0] - one);
    }
    result[result_offset + (NB_MEMORY_COLUMN_PAIRS - 1)] = (next[SORTED_MEMORY_COLUMNS[0].1]
        - current[SORTED_MEMORY_COLUMNS[NB_MEMORY_COLUMN_PAIRS - 1].1])
        * (next[SORTED_MEMORY_COLUMNS[0].0]
            - current[SORTED_MEMORY_COLUMNS[NB_MEMORY_COLUMN_PAIRS - 1].0]
            - one);
}

/// Enforces the permutation argument between offset columns and sorted columns (paper page 60).
fn enforce_offset_aux_constraints<F, E>(
    result: &mut [E],
    result_offset: usize,
    aux_off_offset: usize,
    main_current: &[F],
    main_next: &[F],
    aux_current: &[E],
    aux_next: &[E],
    z: E,
) where
    F: FieldElement,
    E: FieldElement + ExtensionOf<F>,
{
    for i in 0..(NB_OFFSET_COLUMNS - 1) {
        result[result_offset + i] = aux_current[aux_off_offset + i + 1]
            * (z - main_current[SORTED_OFFSET_COLUMNS[i + 1]].into())
            - aux_current[aux_off_offset + i] * (z - main_current[OFFSET_COLUMNS[i + 1]].into());
    }

    result[result_offset + (NB_OFFSET_COLUMNS - 1)] = aux_next[aux_off_offset]
        * (z - main_next[SORTED_OFFSET_COLUMNS[0]].into())
        - aux_current[aux_off_offset + (NB_OFFSET_COLUMNS - 1)]
            * (z - main_next[OFFSET_COLUMNS[0]].into());
}

/// Enforces the permutation argument between memory columns and sorted columns (paper page 60).
fn enforce_memory_aux_constraints<F, E>(
    result: &mut [E],
    result_offset: usize,
    aux_mem_offset: usize,
    main_current: &[F],
    main_next: &[F],
    aux_current: &[E],
    aux_next: &[E],
    z: E,
    alpha: E,
) where
    F: FieldElement,
    E: FieldElement + ExtensionOf<F>,
{
    // Necessary variables: into() fails otherwise. Any better way to do this?
    let mut a: E;
    let mut a2: E;
    for i in 0..(NB_MEMORY_COLUMN_PAIRS - 1) {
        a = main_current[MEMORY_COLUMNS[i + 1].0].into();
        a2 = main_current[SORTED_MEMORY_COLUMNS[i + 1].0].into();
        result[result_offset + i] = aux_current[aux_mem_offset + i + 1]
            * (z - (a2 + alpha * main_current[SORTED_MEMORY_COLUMNS[i + 1].1].into()))
            - aux_current[aux_mem_offset + i]
                * (z - (a + alpha * main_current[MEMORY_COLUMNS[i + 1].1].into()))
    }

    a = main_next[MEMORY_COLUMNS[0].0].into();
    a2 = main_next[SORTED_MEMORY_COLUMNS[0].0].into();
    result[result_offset + (NB_MEMORY_COLUMN_PAIRS - 1)] = aux_next[aux_mem_offset]
        * (z - (a2 + alpha * main_next[SORTED_MEMORY_COLUMNS[0].1].into()))
        - aux_current[aux_mem_offset + (NB_MEMORY_COLUMN_PAIRS - 1)]
            * (z - (a + alpha * main_next[MEMORY_COLUMNS[0].1].into()));
}

// RESCUE HELPER FUNCTIONS
// ------------------------------------------------------------------------------------------------
fn enforce_round<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    result_offset: usize,
    current_state: &[E],
    next_state: &[E],
    round: usize,
) {
    // compute the state that should result from applying the first half of Rescue round
    // to the current state of the computation
    let mut step1 = [E::ZERO; STATE_WIDTH];
    let ark = ARK[round];
    step1.copy_from_slice(current_state);
    apply_sbox(&mut step1);
    apply_mds(&mut step1);
    for i in 0..STATE_WIDTH {
        step1[i] += ark[i].into();
    }

    // compute the state that should result from applying the inverse for the second
    // half for Rescue round to the next step of the computation
    let mut step2 = [E::ZERO; STATE_WIDTH];
    step2.copy_from_slice(next_state);
    for i in 0..STATE_WIDTH {
        step2[i] -= ark[STATE_WIDTH + i].into();
    }
    apply_inv_mds(&mut step2);
    apply_sbox(&mut step2);

    // make sure that the results are equal
    println!("Round {}:\n", round);
    for i in 0..STATE_WIDTH {
        result[result_offset + i] = step2[i] - step1[i];
        println!("step1_{}: {}\nstep2_{}: {}\n", i, step1[i], i, step2[i]);
    }
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_sbox<E: FieldElement>(state: &mut [E]) {
    for i in 0..STATE_WIDTH {
        state[i] = state[i].exp(ALPHA.into());
    }
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_inv_sbox(state: &mut [BaseElement]) {
    for i in 0..STATE_WIDTH {
        state[i] = state[i].exp(INV_ALPHA);
    }
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_mds<E: FieldElement + From<BaseElement>>(state: &mut [E]) {
    let mut result = [E::ZERO; STATE_WIDTH];
    let mut temp = [E::ZERO; STATE_WIDTH];
    for i in 0..STATE_WIDTH {
        for j in 0..STATE_WIDTH {
            temp[j] = E::from(MDS[i * STATE_WIDTH + j]) * state[j];
        }

        for j in 0..STATE_WIDTH {
            result[i] += temp[j];
        }
    }
    state.copy_from_slice(&result);
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_inv_mds<E: FieldElement + From<BaseElement>>(state: &mut [E]) {
    let mut result = [E::ZERO; STATE_WIDTH];
    let mut temp = [E::ZERO; STATE_WIDTH];
    for i in 0..STATE_WIDTH {
        for j in 0..STATE_WIDTH {
            temp[j] = E::from(INV_MDS[i * STATE_WIDTH + j]) * state[j];
        }

        for j in 0..STATE_WIDTH {
            result[i] += temp[j];
        }
    }
    state.copy_from_slice(&result);
}
