use super::{
    rescue, BaseElement, FieldElement, ProofOptions, Prover, PublicInputs, RescueAir, Trace,
    TraceTable, CYCLE_LENGTH, NUM_HASH_ROUNDS,
};

// RESCUE PROVER
// ================================================================================================

pub struct RescueProver {
    options: ProofOptions,
}

impl RescueProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    pub fn build_trace(
        &self,
        seed: [BaseElement; 7],
        iterations: usize,
    ) -> TraceTable<BaseElement> {
        // allocate memory to hold the trace table
        let trace_length = iterations * CYCLE_LENGTH;
        let mut trace = TraceTable::new(14, trace_length);

        trace.fill(
            |state| {
                // initialize first state of the computation
                state[0] = seed[0];
                state[1] = seed[1];
                state[2] = seed[2];
                state[3] = seed[3];
                state[4] = seed[4];
                state[5] = seed[5];
                state[6] = seed[6];
                state[7] = BaseElement::ZERO;
                state[8] = BaseElement::ZERO;
                state[9] = BaseElement::ZERO;
                state[10] = BaseElement::ZERO;
                state[11] = BaseElement::ZERO;
                state[12] = BaseElement::ZERO;
                state[13] = BaseElement::ZERO;
            },
            |step, state| {
                // execute the transition function for all steps
                if (step % CYCLE_LENGTH) < NUM_HASH_ROUNDS {
                    rescue::apply_round(state, step);
                } else {
                    state[7] = BaseElement::ZERO;
                    state[8] = BaseElement::ZERO;
                    state[9] = BaseElement::ZERO;
                    state[10] = BaseElement::ZERO;
                    state[11] = BaseElement::ZERO;
                    state[12] = BaseElement::ZERO;
                    state[13] = BaseElement::ZERO;
                }
            },
        );

        trace
    }
}

impl Prover for RescueProver {
    type BaseField = BaseElement;
    type Air = RescueAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let last_step = trace.length() - 1;
        PublicInputs {
            seed: [
                trace.get(0, 0),
                trace.get(1, 0),
                trace.get(2, 0),
                trace.get(3, 0),
                trace.get(4, 0),
                trace.get(5, 0),
                trace.get(6, 0),
            ],
            result: [
                trace.get(0, last_step),
                trace.get(1, last_step),
                trace.get(2, last_step),
                trace.get(3, last_step),
                trace.get(4, last_step),
                trace.get(5, last_step),
                trace.get(6, last_step),
            ],
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
