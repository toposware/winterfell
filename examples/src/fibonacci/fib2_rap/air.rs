// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::utils::{are_equal, EvaluationResult};
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, ProofOptions, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

// FIBONACCI AIR with homebrewed RAPS
// ================================================================================================
// We add two columns for adding RAPs. We assume the random values used as Challenges in the RAPs are
// independent of the trace (or are the output of Random Oracle evaluated on the trace).

pub const TRACE_WIDTH: usize = 2 + 2 + 1;
pub const TRACE_LENGTH: usize = 64;

pub struct FibRapAir {
    context: AirContext<BaseElement>,
    result: BaseElement,
    rap_challenges: [BaseElement; 2],
}

pub struct PublicInputs {
    pub result: BaseElement,
    pub rap_challenges: [BaseElement; 2],
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.result);
        target.write(&self.rap_challenges[..]);
    }
}

impl Air for FibRapAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
            TransitionConstraintDegree::new(3), // RAPs column 1
            TransitionConstraintDegree::new(3), // RAPs column 2
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]), // equality of RAP columns at the end
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]), // RAPs column 3
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        FibRapAir {
            context: AirContext::new(trace_info, degrees, options),
            result: pub_inputs.result,
            rap_challenges: pub_inputs.rap_challenges,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        _random_coins: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        // expected state width is 2 field elements
        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        let fib_flag = periodic_values[0];
        let step = periodic_values[1];
        let permuted_step = periodic_values[2];
        let eq_flag = periodic_values[3];

        // constraints of Fibonacci sequence (2 terms per step):
        // s_{0, i+1} = s_{0, i} + s_{1, i}
        // s_{1, i+1} = s_{1, i} + s_{0, i+1}
        result.agg_constraint(0, fib_flag, are_equal(next[0], current[0] + current[1]));
        result.agg_constraint(1, fib_flag, are_equal(next[1], current[1] + next[0]));

        // RAPs constraints

        // Single RAP column (attempt)
        // TODO: Clean up afterwards
        enforce_multiset(
            &mut result[5..],
            &current[4..],
            &next[4..],
            compress_tuple(
                vec![next[0], next[1], step],
                E::from(self.rap_challenges[1]),
            ),
            compress_tuple(
                vec![next[0], next[1], permuted_step],
                E::from(self.rap_challenges[1]),
            ),
            E::from(self.rap_challenges[0]),
            E::ONE,
        );

        result[2] = are_equal(
            next[2],
            current[2]
                * (E::from(self.rap_challenges[0])
                    + next[0]
                    + step * E::from(self.rap_challenges[1]))
                * (E::from(self.rap_challenges[0])
                    + next[1]
                    + step * E::from(self.rap_challenges[1])),
        );
        result[3] = are_equal(
            next[3],
            current[3]
                * (E::from(self.rap_challenges[0])
                    + next[0]
                    + permuted_step * E::from(self.rap_challenges[1]))
                * (E::from(self.rap_challenges[0])
                    + next[1]
                    + permuted_step * E::from(self.rap_challenges[1])),
        );
        result.agg_constraint(4, eq_flag, are_equal(next[2], next[3]));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // a valid Fibonacci sequence should start with two ones and terminate with
        // the expected result
        // The last RAP column should start and end as one
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, Self::BaseField::ONE),
            Assertion::single(1, 0, Self::BaseField::ONE),
            Assertion::single(1, last_step, self.result),
            Assertion::single(4, 0, Self::BaseField::ONE),
            Assertion::single(4, last_step, Self::BaseField::ONE),
        ]
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut columns = vec![];
        let mut fib_column = vec![BaseElement::ONE; TRACE_LENGTH];
        fib_column[TRACE_LENGTH / 2 - 2] = BaseElement::ZERO;
        columns.append(&mut vec![fib_column]);

        let steps: Vec<BaseElement> = (1..TRACE_LENGTH + 1)
            .map(|x| BaseElement::from(x as u64))
            .collect();
        let mut permuted_steps = steps.clone();
        columns.append(&mut vec![steps]);
        // note that steps[i] = i + 1. then, to swap TRACE_LENGTH/4-1 with TRACE_LENGTH/2-1 we need to swap indices
        // TRACE_LENGTH/4-2 with TRACE_LENGTH/2-2
        permuted_steps.swap(TRACE_LENGTH / 2 - 2, TRACE_LENGTH / 4 - 2);
        columns.append(&mut vec![permuted_steps]);

        let mut eq_column = vec![BaseElement::ZERO; TRACE_LENGTH];
        eq_column[TRACE_LENGTH - 2] = BaseElement::ONE;
        columns.append(&mut vec![eq_column]);
        columns
    }
}

// MULTISET CHECK
// ==============================================================================================
pub fn enforce_multiset<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    ai: E,
    bi: E,
    gamma: E,
    flag: E,
) {
    // Compute the numerator with ai
    result.agg_constraint(
        0,
        flag,
        are_equal(next[0] * (bi + gamma), current[0] * (ai + gamma)),
    );
}

pub fn compress_tuple<E: FieldElement + From<BaseElement>>(tuple: Vec<E>, alpha: E) -> E {
    let mut element = E::ZERO;
    let mut multiplier = E::ONE;
    for entry in tuple {
        element += multiplier * entry;
        multiplier *= alpha;
    }
    element
}
