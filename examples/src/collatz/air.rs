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

use crate::utils::{are_equal, not, EvaluationResult};

const CYCLE_LENGTH: usize = 2;
// COLLATZ AIR
// ================================================================================================

// TODO 1.2 Choose the right TRACE_WIDTH
//pub(crate) const TRACE_WIDTH: usize = 2;
//pub(crate) const TRACE_WIDTH: usize = 130;
pub(crate) const TRACE_WIDTH: usize = 9;

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
        // TODO 4: You must specify the type and degree of the constraints
        // of your AIR program, based on what you did in TODO 3.
        let mut degrees = vec![];

        // There are constraints for the 129 columns corresponding to the state value and the binary decomposition
        for _ in 1..9 {
            //for _ in 1..258 {
            //for _ in 1..131 {
            degrees.push(TransitionConstraintDegree::with_cycles(
                2,
                vec![CYCLE_LENGTH],
            ));
        }
        //degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        degrees.push(TransitionConstraintDegree::with_cycles(
            1,
            vec![CYCLE_LENGTH],
        ));

        assert_eq!(TRACE_WIDTH, trace_info.width());
        CollatzAir {
            context: AirContext::new(trace_info, degrees, 3, options),
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
        // TODO 2.3 Add the constraints using these periodic values. Dont forget to copy ;-)
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        // expected state width is TRACE_WIDTH field elements large
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // TODO 3: You must enforce some constraints on the trace, to
        // make sure we are indeed checking a proper Collatz sequence.

        let collatz_flag = periodic_values[0];
        // If Collats step, check Collatz
        apply_collatz(result, current, next, collatz_flag);
        // Otherwise, check the binary decomposition
        check_bin_decomp(result, current, next, not(collatz_flag));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // TODO 2: You may define some boundary assertions on the trace.
        //
        // A valid Collatz sequence should start with the provided public
        // input and end with 1, but as we are continuing the sequence
        // to the next power of two, we instead want to enforce that the
        // last term is matching the provided final_input.
        // todo: return a vector of assertions for the first and last step of the program
        let last_step = self.sequence_length - 1;
        //println!("sequence length {}", self.sequence_length);

        vec![
            Assertion::single(0, 0, self.input_value),
            Assertion::single(0, last_step, self.final_value),
            Assertion::single(0, last_step - 1, self.final_value),
        ]
        //unimplemented!();
    }
    // TODO 2.2 In the evaluate_constraints function you don't have access to the step,
    // and hence is not clear how to know wether to enforce collatz or binary_decomp.
    // The way we tell the AIR program what to do is using the peridoc_columns which is just a bunch of
    // vectors whose length divide the trace legth (and hence powers of 2). Whenever the function
    // evaluate_constrains is called for checking the constrains between rows i and i+1, the function
    // receives as input a vector containing periodic_columns[1][i], ..., periodic_columns[n][i]. (Actually
    // 'i' can be any point on the "extended domain").
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        vec![vec![BaseElement::ZERO, BaseElement::ONE]]
        //unimplemented!();
    }
}

fn apply_collatz<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    flag: E,
) {
    // The current state contains the current value as its first element, and the other columns are the binary representation. Check whether the next state's first element is the correct Collatz update.
    //result.agg_constraint(0, flag, are_equal(current[0].div(BaseElement::new(2).into()).mul(current[1].neg().add(BaseElement::new(1).into())).add(current[1].mul(current[0].mul(BaseElement::new(3).into()).add(BaseElement::new(1).into()))), next[0]));
    let collatz1 = (E::ONE - current[1]) * next[0] * BaseElement::new(2).into();
    let collatz2 = current[1] * (E::from(BaseElement::new(3)) * current[0] + E::ONE);
    //let left1 = (E::ONE - current[1]) * collatz1;
    //let left2 = current[1] * collatz2;
    //let left = left1 + left2;
    let left = collatz1 + collatz2;
    let right1 = (E::ONE - current[1]) * current[0];
    let right2 = current[1] * next[0];
    let right = right1 + right2;

    result.agg_constraint(0, flag, are_equal(left, right));

    // Checking that the decomposition columns contain binary elements only
    //for i in 1..129 {
    for i in 1..8 {
        result.agg_constraint(
            i,
            flag,
            are_equal(current[i] * (current[i] - E::ONE), BaseElement::ZERO.into()),
        );
    }
    result.agg_constraint(8, flag, are_equal(current[8], next[8]));
    // for i in 1..129 {
    //     result.agg_constraint(i+128, flag, are_equal(next[0].mul(E::from(BaseElement::ONE).sub(current[1])).add(BaseElement::ONE.into()), next[i]));
    // }
}

fn check_bin_decomp<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    flag: E,
) {
    // We need to check whether the decomposition is correct. For this, we compute the value based on the binary decomposition from the next state and compare it to the value stored in the current state
    let mut initial = next[1];
    //for i in 1..128 {
    for i in 1..7 {
        initial += next[i + 1] * E::from(BaseElement::new((1 as u128) << i));
    }

    // Check that the binary decomp in next state is correct
    result.agg_constraint(0, flag, are_equal(initial, current[0]));
    // assert that next first value = current first value
    result.agg_constraint(1, flag, are_equal(current[0], next[0]));
    //for i in 1..129 {
    for i in 1..8 {
        result.agg_constraint(i + 1, flag, are_equal(current[i], current[8]));
    }
}
