use super::{
    rescue, BaseElement, FieldElement, ProofOptions, Prover, RescueAir, TraceTable, CYCLE_LENGTH,
    NUM_HASH_ROUNDS,
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

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
