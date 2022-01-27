// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    compute_collatz_sequence, BaseElement, CollatzAir, ProofOptions, Prover, PublicInputs,
    StarkField, Trace, TraceTable, TRACE_WIDTH,
};

// COLLATZ PROVER
// ================================================================================================

pub struct CollatzProver {
    options: ProofOptions,
}

impl CollatzProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    /// Builds an execution trace for computing a Collatz sequence.
    pub fn build_trace(
        &self,
        input_value: usize,
        sequence_length: usize,
    ) -> TraceTable<BaseElement> {
        // TODO 1.0: You must construct the trace of the Collatz Air program.
        // This represents the sequence execution at each step, depending on
        // the arity of the current value being processed.
        //
        // Recall that the Collatz sequece is computed by applying sequece_length times the following function
        // to some integer input_value
        //            x/2      if x is even
        //   f(x) =
        //            3x + 1   if x is odd
        //
        // We start considering the function f(b_0, x) where the advice b_0 always indicates if x is odd.
        // The objective is to fill all the trace entries with f(b_00,x_0), f(b_10,f(b_00,x_0)), ... in the
        // way you prefer.
        //
        // Hint: To hint the AIR pogram with the right b_i0 values you simply add them at the right places
        // in the trace.
        //
        // Question: What are the potential risks of assumming that b_i0 is correct?
        //
        // Note: The trace length must be a power of 2.
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // We initialize the execution trace
        let mut trace = TraceTable::new(TRACE_WIDTH, sequence_length);
        trace.fill(
            |state| {
                // todo: initialize the state at step 0
                unimplemented!();
            },
            |_, state| {
                // todo: initialize the state at step i, given the current value (step i-1)
                unimplemented!();
            },
        );

        trace
    }
}

impl Prover for CollatzProver {
    type BaseField = BaseElement;
    type Air = CollatzAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        // TODO 1.1: Depending on how you organized your trace above, you may need to
        // change the line below, which considers that you wrote the input value of the
        // Collatz sequence in step (row) 0, column 0.
        let input_value = trace.get(0, 0);
        let sequence_length = trace.length();
        let final_value = BaseElement::from(compute_collatz_sequence(
            input_value.to_repr() as usize,
            sequence_length,
        ) as u128);

        PublicInputs {
            input_value,
            final_value,
            sequence_length,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
