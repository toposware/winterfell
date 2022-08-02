// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

// DIVISORS EXEMPTIONS AIR
// ================================================================================================

// Public inputs contain the two results and the position of the exponentiation result
#[derive(Clone)]
pub struct PublicInputs {
    pub results: [BaseElement; 2],
    pub last_exp_step: u64,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(&self.results[..]);
        target.write_u64(self.last_exp_step);
    }
}

pub struct DivisorsExemptionsAir {
    context: AirContext<BaseElement>,
    result: PublicInputs,
}

impl Air for DivisorsExemptionsAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut divisors = Vec::new();

        // Set extra divisors used in the AIR. Each custom divisor is described by a set of
        // exemptions points.
        // TODO: [divisors] change description of divisor to include additionally cosets
        let custom_divisor = trace_info.length() + 1 - pub_inputs.last_exp_step as usize;
        divisors.push(custom_divisor);

        // Overwrite main and auxiliary constraint divisors. Each constraint is paired with
        // an index that points in one of the divisors. The first divisor is the default divisor.
        let main_constraint_divisors: Vec<usize> = Vec::from([0, 0, 1]);
        let aux_constraint_divisors: Vec<usize> = Vec::new();

        DivisorsExemptionsAir {
            context: AirContext::new(trace_info, degrees, 5, options).set_custom_divisors(
                divisors,
                main_constraint_divisors,
                aux_constraint_divisors,
            ),
            result: pub_inputs,
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
        let current = frame.current();
        let next = frame.next();
        // expected state width is 3 field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // TODO: [divisors] create different divisors for examples

        // constraints of Fibonacci sequence (2 terms per step):
        // s_{0, i+1} = s_{0, i} + s_{1, i}
        // s_{1, i+1} = s_{1, i} + s_{0, i+1}
        result[0] = are_equal(next[0], current[0] + current[1]);
        result[1] = are_equal(next[1], current[1] + next[0]);
        // constraints of exponentiation sequence:
        result[2] = are_equal(next[2], current[2] + current[2]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_fib_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, Self::BaseField::ONE),
            Assertion::single(1, 0, Self::BaseField::ONE),
            Assertion::single(1, last_fib_step, self.result.results[0]),
            Assertion::single(2, 0, Self::BaseField::ONE),
            Assertion::single(
                2,
                self.result.last_exp_step as usize - 1,
                self.result.results[1],
            ),
        ]
    }
}
