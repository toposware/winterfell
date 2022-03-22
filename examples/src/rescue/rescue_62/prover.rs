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
        seed: [BaseElement; 2],
        iterations: usize,
        width: usize,
    ) -> TraceTable<BaseElement> {
        // allocate memory to hold the trace table
        let trace_length = iterations * CYCLE_LENGTH;
        let mut trace = TraceTable::new(width, trace_length);
        let steps = width / 4;

        trace.fill(
            |state| {
                // initialize first state of the computation
                for i in 0..steps {
                    state[4 * i] = seed[0];
                    state[4 * i + 1] = seed[1];
                    state[4 * i + 2] = BaseElement::ZERO;
                    state[4 * i + 3] = BaseElement::ZERO;
                }
            },
            |step, state| {
                // execute the transition function for all steps
                //
                // for the first 14 steps in every cycle, compute a single round of
                // Rescue hash; for the remaining 2 rounds, just carry over the values
                // in the first two registers to the next step
                if (step % CYCLE_LENGTH) < NUM_HASH_ROUNDS {
                    for i in 0..steps {
                        rescue::apply_round(&mut state[4 * i..4 * i + 4], step);
                    }
                } else {
                    for i in 0..steps {
                        state[4 * i + 2] = BaseElement::ZERO;
                        state[4 * i + 3] = BaseElement::ZERO;
                    }
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
            seed: [trace.get(0, 0), trace.get(1, 0)],
            result: [trace.get(0, last_step), trace.get(1, last_step)],
            width: trace.width(),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
