// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo,
    TransitionConstraintDegree,
};

pub struct PublicInputs {
    pub input: BaseElement,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.input);
    }
}

// VIRTUAL COLUMN MINIMAL AIR
// ================================================================================================

pub struct VCMinimalAir {
    context: AirContext<BaseElement>,
    pub_inputs: PublicInputs,
}

impl Air for VCMinimalAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees =
            vec![TransitionConstraintDegree::new(2); trace_info.layout().virtual_trace_width()];
        //assert_eq!(TRACE_WIDTH, trace_info.layout().main_trace_width());
        let context = AirContext::new(trace_info, degrees, 1, options);
        VCMinimalAir {
            context,
            pub_inputs,
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

        let width = self.trace_layout().virtual_trace_width();
        for i in 0..width - 1 {
            result[i] = are_equal(current[i + 1], current[i].square());
        }
        result[width - 1] = are_equal(next[0], current[width - 1].square());
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Add a dummy assetion for now.
        vec![Assertion::single(0, 0, self.pub_inputs.input)]
    }
}
