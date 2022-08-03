// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, DivisorsExemptionsAir, FieldElement, ProofOptions, Prover, Trace, TraceTable,
    TRACE_WIDTH,
};

use crate::utils::print_trace;

// DIVISORS EXEMPTIONS PROVER
// ===============================================================================================

pub struct DivisorsExemptionsProver {
    options: ProofOptions,
    // The last exponentiation step cannot be deduced from the trace since it can have exemptions.
    // We give it explicitely as an input to the prover to get the corresponding exemption.
    last_exp_step: u64,
}

impl DivisorsExemptionsProver {
    pub fn new(options: ProofOptions, last_exp_step: u64) -> Self {
        Self {
            options,
            last_exp_step,
        }
    }

    /// Builds an execution trace for making two computations in parallel:
    /// (1) computing a Fibonacci sequence of the specified length such that each row advances the sequence by 2 terms,
    /// (2) computing powers of two for a specified number of times
    pub fn build_trace(&self, sequence_length: [u64; 2]) -> TraceTable<BaseElement> {
        assert!(
            sequence_length[0].is_power_of_two(),
            "fib sequence length must be a power of 2"
        );
        assert!(
            sequence_length[1] <= sequence_length[0] / 2,
            "exp sequence length must be at most half of sequence length"
        );

        let mut trace = TraceTable::new(TRACE_WIDTH, sequence_length[0] as usize / 2);
        trace.fill(
            |state| {
                state[0] = BaseElement::ONE;
                state[1] = BaseElement::ONE;
                state[2] = BaseElement::ONE;
            },
            |i, state| {
                state[0] += state[1];
                state[1] += state[0];
                // We stop doubling at this point to force the AIR constraint to fail in the last steps if we use
                // the "standard" divisor
                if i < sequence_length[1] as usize - 1 {
                    state[2] += state[2];
                } else {
                    // arbitrary values for the exempted points
                    state[2] = BaseElement::ZERO;
                }
            },
        );

        print_trace(&trace, 1, 0, 0..3);
        trace
    }
}

impl Prover for DivisorsExemptionsProver {
    type BaseField = BaseElement;
    type Air = DivisorsExemptionsAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> super::air::PublicInputs {
        let last_fib_step = trace.length() - 1;
        let last_exp_step = self.last_exp_step - 1;
        // two public inputs: results for each exponentiations
        let results = [
            trace.get(1, last_fib_step),
            trace.get(2, last_exp_step as usize),
        ];
        super::air::PublicInputs {
            results,
            last_exp_step: self.last_exp_step,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
