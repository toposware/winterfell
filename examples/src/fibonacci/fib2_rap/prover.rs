// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::super::utils::compute_fib_term;
use super::{
    compress_tuple, BaseElement, FibRapAir, FieldElement, ProofOptions, Prover, PublicInputs,
    Trace, TraceTable, TRACE_LENGTH, TRACE_WIDTH,
};

// FIBONACCI RAP PROVER
// ================================================================================================

pub struct FibRapProver {
    options: ProofOptions,
}

impl FibRapProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    /// Builds an execution trace for computing a Fibonacci sequence of the specified length such
    /// that each row advances the sequence by 2 terms.
    pub fn build_trace(&self, sequence_length: usize) -> TraceTable<BaseElement> {
        assert_eq!(TRACE_LENGTH, sequence_length / 2, "No wei hemano");
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );
        let mut trace = TraceTable::new(TRACE_WIDTH, 3, TRACE_LENGTH, 2);
        trace.fill(
            |state| {
                state[0] = BaseElement::ONE;
                state[1] = BaseElement::ONE;
            },
            |step, state| {
                state[0] += state[1];
                state[1] += state[0];

                if step == TRACE_LENGTH / 2 - 2 {
                    // Copy previous state
                    state[0] = compute_fib_term(2 * TRACE_LENGTH / 4 - 1);
                    state[1] = compute_fib_term(2 * TRACE_LENGTH / 4);
                }
            },
        );

        trace.fill_aux(
            |rap_challenges, state_init, state_aux| {
                state_aux[0] = (rap_challenges[0]
                    + state_init[0]
                    + BaseElement::from(0u64) * rap_challenges[1])
                    * (rap_challenges[0]
                        + state_init[1]
                        + BaseElement::from(0u64) * rap_challenges[1]);
                state_aux[1] = (rap_challenges[0]
                    + state_init[0]
                    + BaseElement::from(0u64) * rap_challenges[1])
                    * (rap_challenges[0]
                        + state_init[1]
                        + BaseElement::from(0u64) * rap_challenges[1]);
                state_aux[2] = BaseElement::ONE;
            },
            |step, rap_challenges, state_init, state_aux| {
            // Add a copy constraint between the sequence_length/4 and sequence_length/2 rows  using raps
            // step is pointing to the previous row w.r.t. the want we want to copy.
            let mut permuted_step = step;
            if step == TRACE_LENGTH/4 - 2 {
                let st0 = compute_fib_term(2*TRACE_LENGTH/4 - 1);
                let st1 = compute_fib_term(2*TRACE_LENGTH/4);
                permuted_step = TRACE_LENGTH / 2 - 2;
                assert_eq!(
                    state_init[0], st0,
                    "At step {} state[0] = {} while compute_fib_term({}) = {}. And btw compute_fib_term({}) = {}",
                    step + 1, state_init[0], 2*TRACE_LENGTH/4 - 1, st0,
                    3, compute_fib_term(3));
                assert_eq!(
                    state_init[1], st1,
                    "At step {} state[1] = {} while compute_fib_term({}) = {}",
                    step + 1, state_init[1], 2*TRACE_LENGTH/4, st1);
                state_aux[0] = state_aux[0]*
                (rap_challenges[0] + state_init[0]  + BaseElement::from((step + 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((step + 1) as u64)*rap_challenges[1]);
                state_aux[1] = state_aux[1]*
                (rap_challenges[0] + state_init[0] + BaseElement::from((TRACE_LENGTH/2 - 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((TRACE_LENGTH/2 - 1) as u64)*rap_challenges[1]);
            }
            else if step == TRACE_LENGTH/2 - 2 {
                permuted_step = TRACE_LENGTH / 4 - 2;
                state_aux[0] = state_aux[0]*
                (rap_challenges[0] + state_init[0]  + BaseElement::from((step + 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((step + 1) as u64)*rap_challenges[1]);
                state_aux[1] = state_aux[1]*
                (rap_challenges[0] + state_init[0] + BaseElement::from((TRACE_LENGTH/4 - 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((TRACE_LENGTH/4 - 1) as u64)*rap_challenges[1]);
            }
            else {
                state_aux[0] = state_aux[0]*
                (rap_challenges[0] + state_init[0]  + BaseElement::from((step + 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((step + 1) as u64)*rap_challenges[1]);
                state_aux[1] = state_aux[1]*
                (rap_challenges[0] + state_init[0] + BaseElement::from((step + 1) as u64)*rap_challenges[1])
                *(rap_challenges[0] + state_init[1] + BaseElement::from((step + 1) as u64)*rap_challenges[1]);
            }
            let state0 = state_init[0];
            let state1 = state_init[1];
            apply_multiset(
                &mut state_aux[2..],
                compress_tuple(
                    vec![state0, state1, BaseElement::from((step + 1) as u64)],
                    rap_challenges[1],
                ),
                compress_tuple(
                    vec![state0, state1, BaseElement::from((permuted_step + 1) as u64)],
                    rap_challenges[1],
                ),
                rap_challenges[0]);
            }
        );

        trace
    }
}

impl Prover for FibRapProver {
    type BaseField = BaseElement;
    type Air = FibRapAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let last_step = trace.length() - 1;

        PublicInputs {
            result: trace.get(1, last_step),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

pub fn apply_multiset<E: FieldElement + From<BaseElement>>(
    state: &mut [E],
    ai: E,
    bi: E,
    gamma: E,
) {
    // Compute the numerator with ai
    state[0] *= (ai + gamma) / (bi + gamma);
}
