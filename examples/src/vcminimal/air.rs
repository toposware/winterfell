// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, FieldElement, ProofOptions, TRACE_WIDTH};
use crate::utils::are_equal;
use winterfell::{
    Air, AirContext, Assertion, ByteWriter, EvaluationFrame, Serializable, TraceInfo, TransitionConstraintDegree,
};

pub struct PublicInputs {
    pub input: [BaseElement; 2],
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(&self.input[..]);
    }
}

// VIRTUAL COLUMN MINIMAL AIR
// ================================================================================================

pub struct VCMinimalAir {
    context: AirContext<BaseElement>,
    pub_inputs: PublicInputs
}

impl Air for VCMinimalAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![TransitionConstraintDegree::new(2); 4];
        // ALEX: change for virtual width
        //assert_eq!(TRACE_WIDTH, trace_info.layout().virtual_trace_width());
        assert_eq!(TRACE_WIDTH, trace_info.layout().main_trace_width());
        let context =
            AirContext::new(
                trace_info, 
                degrees,
                2,
                options);
        VCMinimalAir {
            context,
            pub_inputs
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
        let two = E::from(2u128);

        let current = frame.current();
        let next = frame.next();

        // Check ap correctness at result[0]
        result[0] = are_equal(next[0], current[0]*current[0]);
        result[1] = are_equal(next[1], current[1]*current[1]);
        result[2] = are_equal(next[2], two*current[0]*current[1]);
        result[3] = are_equal(next[3], (current[0]+current[1])*(current[0]+current[1]));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // Add a dummy assetion for now.
        vec![
            Assertion::single(0, 0, self.pub_inputs.input[0]),
            Assertion::single(1, 0, self.pub_inputs.input[1])
        ]
    }
}
