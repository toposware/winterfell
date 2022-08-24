// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    BaseElement, DivisorsCosetsAir, FieldElement, ProofOptions, Prover, Trace, TraceTable,
    TRACE_WIDTH,
};

use crate::utils::print_trace;
use rand::Rng;

// DIVISORS COSETS PROVER
// ===============================================================================================

pub struct DivisorsCosetsProver {
    options: ProofOptions,
    range_length: u64,
    offset: u64,
}

impl DivisorsCosetsProver {
    pub fn new(options: ProofOptions, range_length: u64, offset: u64) -> Self {
        Self {
            options,
            range_length,
            offset,
        }
    }

    /// Builds an execution trace for exponentiating two to some power. Additionally, in a second raw, it performs a check that
    /// (some) values are bit. We do not care where these elements come from at the moment.
    pub fn build_trace(
        &self,
        sequence_length: u64,
        range_length: u64,
        offset: u64,
    ) -> TraceTable<BaseElement> {
        assert!(
            sequence_length.is_power_of_two(),
            "sequence length must be a power of 2"
        );
        assert!(
            range_length.is_power_of_two() && range_length <= sequence_length,
            "range sequence length must be a power of 2 and smaller or equal to exp sequence length"
        );
        assert!(
            offset < sequence_length / range_length,
            "offset should be greater than 0 and smaller than sequence_length/range_length"
        );

        // sample random bits to fill the second column. Each bit represent the flag and in the next
        // computational step the flag should be reversed.
        let mut rng = rand::thread_rng();
        let mut bits = Vec::<u128>::new();
        // get random numbers
        for _ in 0..range_length {
            let value = rng.gen_range(0..=1u128);
            let next_value = 1 - value;
            bits.push(value);
            bits.push(next_value);
            for _ in 0..(sequence_length / range_length - 2) {
                bits.push(rng.gen_range(2..=1000u128));
            }
        }
        println!("{:?}", bits);

        // TODO: [divisors] fill only the correct places with bit to test custom divisors
        let mut trace = TraceTable::new(TRACE_WIDTH, sequence_length as usize);
        trace.fill(
            |state| {
                state[0] = BaseElement::ONE;
                state[1] = BaseElement::new(
                    bits[(sequence_length as usize - offset as usize) % sequence_length as usize],
                );
            },
            |i, state| {
                let idx =
                    (sequence_length as usize + i + 1 - offset as usize) % sequence_length as usize;
                state[0] += state[0];
                state[1] = BaseElement::new(bits[idx]);
            },
        );

        print_trace(&trace, 1, 0, 0..2);
        trace
    }
}

impl Prover for DivisorsCosetsProver {
    type BaseField = BaseElement;
    type Air = DivisorsCosetsAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> super::air::PublicInputs {
        let last_step = trace.length() - 1;
        let result = trace.get(0, last_step);
        super::air::PublicInputs {
            result,
            range_length: self.range_length,
            offset: self.offset,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
