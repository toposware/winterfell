use super::{
    rescue, BaseElement, ExecutionTrace, FieldElement, ProofOptions, Prover, RescueAir,
    CYCLE_LENGTH, NUM_HASH_ROUNDS,
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
        seed: [BaseElement; 2],
        iterations: usize,
    ) -> ExecutionTrace<BaseElement> {
        // allocate memory to hold the trace table
        let trace_length = iterations * CYCLE_LENGTH;
        let mut trace = ExecutionTrace::new(4, trace_length);

        trace.fill(
            |state| {
                // initialize first state of the computation
                state[0] = seed[0];
                state[1] = seed[1];
                state[2] = BaseElement::ZERO;
                state[3] = BaseElement::ZERO;
            },
            |step, state| {
                // execute the transition function for all steps
                //
                // for the first 14 steps in every cycle, compute a single round of
                // Rescue hash; for the remaining 2 rounds, just carry over the values
                // in the first two registers to the next step
                if (step % CYCLE_LENGTH) < NUM_HASH_ROUNDS {
                    rescue::apply_round(state, step);
                } else {
                    state[2] = BaseElement::ZERO;
                    state[3] = BaseElement::ZERO;
                }
            },
        );

        trace
    }
}

impl Prover for RescueProver {
    type BaseField = BaseElement;
    type Air = RescueAir;
    type Trace = ExecutionTrace<BaseElement>;

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
