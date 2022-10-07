// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::math::FieldElement;


use rand::Rng;

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
        //
        // PART 2 Now we will not assume that the parity bit is correctly computed. To do so,
        // we are forced to non-deterministically guess the whole binary representation and then
        // check its correctness. (Hard question: Can we prove that it is necessary to check the whole
        // binary representation even when we just need the parity bit?). To do so, our AIR program needs
        // to switch between two states: a) non-deterministically checking the validity of the binary decomposition;
        // and b) enforcing the correct computation of the Collatz function repeating the following patern
        // +---------------+
        // | binary_decomp |
        // +---------------+
        // | collatz       |
        // +---------------+
        // QUESTION 2. What other pattern could we use?
        // ATENTION! This time we will use only the aforementioned pattern.unimplemented
        //
        // TODO 2.1 Compute the trace as mentioned before. Note that now the function for updating
        // the trace rows receives the step as input. Use that value to know wether you need to
        // do collatz or the binary decomposition.
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // We initialize the execution trace
        let mut trace = TraceTable::new(TRACE_WIDTH, sequence_length);
        println!("while sequence {}", sequence_length);
        trace.fill(
            |state| {
                // todo: initialize the state at step 0

                // Initialize with input value and bit representation of the input value (first step = binary decomposition)
                state[0] = BaseElement::new(input_value as u128);
                //for i in 1..128 {
                let guess = state[0].to_repr();
                for i in 1..129 {
                    state[i] = BaseElement::new(guess>>(i-1) & (1 as u128));
                    //println!("state init n°{} {}", i, state[i]);
                }
                let mut rng = rand::thread_rng();
                let init_random: u128 = rng.gen();
                //state[129] = init_random.into();
                state[129] = init_random.into();
                println!("init state {}", state[0]);
            },
            |i, state| {
                if i % 2 == 1 {
                    let guess = state[0].to_repr(); // get representation of the previous step as a u128
                    // fill the binary representation
                    for j in 0..128 {
                        state[j+1] = BaseElement::new(guess>>j & (1 as u128));
                    }
                    let mut rng = rand::thread_rng();
                    let init_random: u128 = rng.gen();
                    //state[129] = init_random.into();
                    state[129] = init_random.into();
                } else {
                    // Collatz step
                    if state[1] == BaseElement::ZERO {
                        state[0] = state[0]/BaseElement::new(2);
                    } else {
                        state[0] = BaseElement::new(3) * state[0] + BaseElement::new(1);
                    }

                    // If we set the intermediary values to 0 or keep them unchanged from the previous row, there is a high risk that there will be columns with only 0. So I added this as a way to prevent all-0 columns
                    // for j in 2..129 {
                    //     state[j] = state[0] * (BaseElement::ONE - state[1]) + BaseElement::ONE;
                    // }
                    // state[1] = state[0] * (BaseElement::ONE - state[1]) + BaseElement::ONE;
                    // let mut rng = rand::thread_rng();
                    // let random_nb: u128 = rng.gen();
                    // for j in 1..130 {
                    //     state[j] = random_nb.into();
                    // }
                    for j in 1..129 {
                        state[j] = state[129];
                    }
                }
                // todo: initialize the state at step i, given the current value (step i-1)
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
        println!("trace length {}", sequence_length);
        let final_value = BaseElement::from(compute_collatz_sequence(
            input_value.to_repr() as usize,
            sequence_length/2+1,
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
