// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree,
};

pub struct PublicInputs {}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {    }
}

// FIBONACCI AIR
// ================================================================================================

pub struct DegreeProblemAir {
    context: AirContext<BaseElement>
}

impl Air for DegreeProblemAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![TransitionConstraintDegree::new(2); 4];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let context = AirContext::new(trace_info, degrees, 1, options);
        DegreeProblemAir {
            context,
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
        for i in 0..4 {
            result[i] = current[i] * (current[i] - E::ONE);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Add a dummy assertion.
        vec![
            Assertion::single(0, 0, Self::BaseField::ZERO)
        ]
    }
}
