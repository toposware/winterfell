// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    compute_collatz_sequence, BaseElement, CollatzAir, ProofOptions, Prover, PublicInputs,
    StarkField, Trace, TraceTable,
};

use crate::utils::print_trace;

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
        // The Collatz sequece is computed by applying sequece_length times the following function
        // to some integer input_value
        //            x/2      if x is even
        //   f(x) =
        //            3x + 1   if x is odd
        //
        // We start considering the function f(b_0, x) where the advice b_0 always indicates if x is odd.
        //
        // We non-deterministically guess the whole binary representation and then
        // check its correctness. To do so, our AIR program needs
        // to switch between two states: a) non-deterministically checking the validity of the binary decomposition;
        // and b) enforcing the correct computation of the Collatz function repeating the following patern
        // +---------------+
        // | binary_decomp |
        // +---------------+
        // | collatz       |
        // +---------------+
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );

        // Next number in sequence
        let next = |x| {
            if x % 2 == 0 {
                x / 2
            } else {
                3 * x + 1
            }
        };

        // decomposition of a number in bits
        let division = |x| {
            let mut div: u128 = x;
            let mut decomposition = vec![];

            for _ in 0..128 {
                decomposition.push((div / 2, div % 2));
                div = div / 2;
            }
            decomposition
        };

        let mut sequence = vec![];
        let mut current = input_value as u128;

        // compute the collatz sequence
        for _ in 0..sequence_length {
            sequence.push(current);
            current = next(current);
        }

        // decomposition values for each element in the sequence
        let decomposition_values = sequence.iter().map(|x| division(*x)).collect::<Vec<_>>();

        let mut trace = TraceTable::new(4, 128 * sequence_length);
        trace.fill(
            |state| {
                state[0] = BaseElement::new(sequence[0]);
                state[1] = BaseElement::new(decomposition_values[0][0].1);
                state[2] = BaseElement::new(decomposition_values[0][0].0);
                state[3] = BaseElement::new(decomposition_values[0][0].1);
            },
            |i, state| {
                state[0] = BaseElement::new(sequence[(i + 1) / 128]);
                state[1] = BaseElement::new(decomposition_values[(i + 1) / 128][0].1);
                state[2] = BaseElement::new(decomposition_values[(i + 1) / 128][(i + 1) % 128].0);
                state[3] = BaseElement::new(decomposition_values[(i + 1) / 128][(i + 1) % 128].1);
            },
        );

        // print_trace(&trace, 1, 0, 0..4);
        trace
    }
}

impl Prover for CollatzProver {
    type BaseField = BaseElement;
    type Air = CollatzAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let input_value = trace.get(0, 0);
        let sequence_length = trace.length() / 128;
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
