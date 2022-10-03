// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, ProofOptions, Serializable, TraceInfo,
};

// COLLATZ AIR
// ================================================================================================

// TODO 1.2 Choose the right TRACE_WIDTH
//pub(crate) const TRACE_WIDTH: usize = 2;
pub(crate) const TRACE_WIDTH: usize = 128;

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
        let degrees = vec![];
        let num_assertions = 2; // to change
        assert_eq!(TRACE_WIDTH, trace_info.width());
        CollatzAir {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
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

        // todo: update the result slice with constraints related to
        // the Collatz conjecture
        unimplemented!();
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // TODO 2: You may define some boundary assertions on the trace.
        //
        // A valid Collatz sequence should start with the provided public
        // input and end with 1, but as we are continuing the sequence
        // to the next power of two, we instead want to enforce that the
        // last term is matching the provided final_input.
        // todo: return a vector of assertions for the first and last step of the program
        unimplemented!();
    }

    // TODO 2.2 In the evaluate_constraints function you don't have access to the step,
    // and hence is not clear how to know wether to enforce collatz or binary_decomp.
    // The way we tell the AIR program what to do is using the peridoc_columns which is just a bunch of
    // vectors whose length divide the trace legth (and hence powers of 2). Whenever the function
    // evaluate_constrains is called for checking the constrains between rows i and i+1, the function
    // receives as input a vector containing periodic_columns[1][i], ..., periodic_columns[n][i]. (Actually
    // 'i' can be any point on the "extended domain").
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        unimplemented!();
    }
}
