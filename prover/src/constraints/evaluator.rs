// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{
    super::TraceLde, evaluation_table::EvaluationTableFragment, BoundaryConstraints,
    ConstraintEvaluationTable, PeriodicValueTable, StarkDomain,
};
use air::{
    Air, AuxTraceRandElements, ConstraintCompositionCoefficients, EvaluationFrame,
    TransitionConstraints,
};
use math::FieldElement;
use utils::{iter_mut, collections::BTreeMap};

#[cfg(feature = "concurrent")]
use utils::{iterators::*, rayon};

// CONSTANTS
// ================================================================================================

#[cfg(feature = "concurrent")]
const MIN_CONCURRENT_DOMAIN_SIZE: usize = 8192;

// CONSTRAINT EVALUATOR
// ================================================================================================

pub struct ConstraintEvaluator<'a, A: Air, E: FieldElement<BaseField = A::BaseField>> {
    air: &'a A,
    boundary_constraints: BoundaryConstraints<E>,
    transition_constraints: TransitionConstraints<E>,
    aux_rand_elements: AuxTraceRandElements<E>,
    periodic_values: PeriodicValueTable<E::BaseField>,
}

impl<'a, A: Air, E: FieldElement<BaseField = A::BaseField>> ConstraintEvaluator<'a, A, E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new evaluator which can be used to evaluate transition and boundary constraints
    /// over extended execution trace.
    pub fn new(
        air: &'a A,
        aux_rand_elements: AuxTraceRandElements<E>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self {
        // build transition constraint groups; these will be used to compose transition constraint
        // evaluations
        let transition_constraints =
            air.get_transition_constraints(&composition_coefficients.transition);

        // build periodic value table
        let periodic_values = PeriodicValueTable::new(air);

        // build boundary constraint groups; these will be used to evaluate and compose boundary
        // constraint evaluations.
        let boundary_constraints =
            BoundaryConstraints::new(air, &aux_rand_elements, &composition_coefficients.boundary);

        ConstraintEvaluator {
            air,
            boundary_constraints,
            transition_constraints,
            aux_rand_elements,
            periodic_values,
        }
    }

    // EVALUATOR
    // --------------------------------------------------------------------------------------------
    /// Evaluates constraints against the provided extended execution trace. Constraints are
    /// evaluated over a constraint evaluation domain. This is an optimization because constraint
    /// evaluation domain can be many times smaller than the full LDE domain.
    pub fn evaluate(
        self,
        trace: &TraceLde<E>,
        domain: &StarkDomain<E::BaseField>,
    ) -> ConstraintEvaluationTable<E> {
        assert_eq!(
            trace.trace_len(),
            domain.lde_domain_size(),
            "extended trace length is not consistent with evaluation domain"
        );

        // build a list of constraint divisors; we first put transition constraints at the beggining
        // of the list and then append  boundary constraint divisors         
        let mut divisors = vec![];
        for divisor in self.transition_constraints.divisors() {
            divisors.push(divisor.clone());
        }
        for divisor in self.boundary_constraints.get_divisors() {
            divisors.push(divisor.clone());
        }

        // allocate space for constraint evaluations; when we are in debug mode, we also allocate
        // memory to hold all transition constraint evaluations (before they are merged into a
        // single value) so that we can check their degrees later
        #[cfg(not(debug_assertions))]
        let mut evaluation_table = ConstraintEvaluationTable::<E>::new(domain, divisors);
        #[cfg(debug_assertions)]
        let mut evaluation_table =
            ConstraintEvaluationTable::<E>::new(domain, divisors, &self.transition_constraints);

        // when `concurrent` feature is enabled, break the evaluation table into multiple fragments
        // to evaluate them into multiple threads; unless the constraint evaluation domain is small,
        // then don't bother with concurrent evaluation

        #[cfg(not(feature = "concurrent"))]
        let num_fragments = 1;

        #[cfg(feature = "concurrent")]
        let num_fragments = if domain.ce_domain_size() >= MIN_CONCURRENT_DOMAIN_SIZE {
            rayon::current_num_threads().next_power_of_two()
        } else {
            1
        };

        // evaluate constraints for each fragment; if the trace consist of multiple segments
        // we evaluate constraints for all segments. otherwise, we evaluate constraints only
        // for the main segment.
        let mut fragments = evaluation_table.fragments(num_fragments);
        iter_mut!(fragments).for_each(|fragment| {
            if self.air.trace_info().is_multi_segment() {
                self.evaluate_fragment_full(trace, domain, fragment);
            } else {
                self.evaluate_fragment_main(trace, domain, fragment);
            }
        });

        // when in debug mode, make sure expected transition constraint degrees align with
        // actual degrees we got during constraint evaluation
        #[cfg(debug_assertions)]
        evaluation_table.validate_transition_degrees();

        evaluation_table
    }

    // EVALUATION HELPERS
    // --------------------------------------------------------------------------------------------

    /// Evaluates constraints for a single fragment of the evaluation table.
    ///
    /// This evaluates constraints only over the main segment of the execution trace.
    fn evaluate_fragment_main(
        &self,
        trace: &TraceLde<E>,
        domain: &StarkDomain<A::BaseField>,
        fragment: &mut EvaluationTableFragment<E>,
    ) {
        // initialize buffers to hold trace values and evaluation results at each step;
        let mut main_frame = EvaluationFrame::new(trace.main_trace_width());
        let mut evaluations = vec![E::ZERO; fragment.num_columns()];
        let mut t_evaluations = vec![E::BaseField::ZERO; self.num_main_transition_constraints()];

        // pre-compute values needed to determine x coordinates in the constraint evaluation domain
        let g = domain.ce_domain_generator();
        let mut x = domain.offset() * g.exp((fragment.offset() as u64).into());

        // this will be used to convert steps in constraint evaluation domain to steps in
        // LDE domain
        let lde_shift = domain.ce_to_lde_blowup().trailing_zeros();

        for i in 0..fragment.num_rows() {
            let step = i + fragment.offset();

            // update evaluation frame buffer with data from the execution trace; this will
            // read current and next rows from the trace into the buffer; data in the trace
            // table is extended over the LDE domain, so, we need to convert step in constraint
            // evaluation domain, into a step in LDE domain, in case these domains are different
            trace.read_main_trace_frame_into(step << lde_shift, &mut main_frame);

            // evaluate transition constraints per divisor and save the merged results at the first 
            // l slot of the evaluations buffer where l is the number of transition constraint
            // divisors used
            let main_evals = self.evaluate_main_transition(&main_frame, x, step, &mut t_evaluations);
            for i in 0..main_evals.len() {
                evaluations[i] = main_evals[i];
            }
            // when in debug mode, save transition constraint evaluations
            #[cfg(debug_assertions)]
            fragment.update_transition_evaluations(step, &t_evaluations, &[]);

            // evaluate boundary constraints; the results go into remaining slots of the
            // evaluations buffer
            let main_state = main_frame.current();
            self.boundary_constraints
                .evaluate_main(main_state, x, step, &mut evaluations[main_evals.len()..]);

            // record the result in the evaluation table
            fragment.update_row(i, &evaluations);

            // update x to the next value
            x *= g;
        }
    }

    /// Evaluates constraints for a single fragment of the evaluation table.
    ///
    /// This evaluates constraints only over all segments of the execution trace (i.e. main segment
    /// and all auxiliary segments).
    fn evaluate_fragment_full(
        &self,
        trace: &TraceLde<E>,
        domain: &StarkDomain<A::BaseField>,
        fragment: &mut EvaluationTableFragment<E>,
    ) {
        // initialize buffers to hold trace values and evaluation results at each step
        let mut main_frame = EvaluationFrame::new(trace.main_trace_width());
        let mut aux_frame = EvaluationFrame::new(trace.aux_trace_width());
        let mut tm_evaluations = vec![E::BaseField::ZERO; self.num_main_transition_constraints()];
        let mut ta_evaluations = vec![E::ZERO; self.num_aux_transition_constraints()];
        let mut evaluations = vec![E::ZERO; fragment.num_columns()];

        // pre-compute values needed to determine x coordinates in the constraint evaluation domain
        let g = domain.ce_domain_generator();
        let mut x = domain.offset() * g.exp((fragment.offset() as u64).into());

        // this will be used to convert steps in constraint evaluation domain to steps in
        // LDE domain
        let lde_shift = domain.ce_to_lde_blowup().trailing_zeros();

        for i in 0..fragment.num_rows() {
            let step = i + fragment.offset();

            // read both the main and the auxiliary evaluation frames from the trace
            trace.read_main_trace_frame_into(step << lde_shift, &mut main_frame);
            trace.read_aux_trace_frame_into(step << lde_shift, &mut aux_frame);

            // evaluate transition constraints per divisor and save the merged results at the first 
            // l slot of the evaluations buffer where l is the number of transition constraint
            // divisors used;  we evaluate and compose constraints in the same function, we
            // can just add up the results of evaluating main and auxiliary constraints.
            let main_evals = self.evaluate_main_transition(&main_frame, x, step, &mut tm_evaluations);
            for i in 0..main_evals.len() {
                evaluations[i] = main_evals[self.transition_constraints.main_constraints_divisors()[i]];
            }

            let aux_evals = self.evaluate_aux_transition(&main_frame, &aux_frame, x, step, &mut ta_evaluations);
            for i in 0..aux_evals.len() {
                evaluations[i] += aux_evals[i];
            }

            // TODO [divisors]: restore assertion
            // when in debug mode, save transition constraint evaluations
            #[cfg(debug_assertions)]
            fragment.update_transition_evaluations(step, &tm_evaluations, &ta_evaluations);

            // evaluate boundary constraints; the results go into remaining slots of the
            // evaluations buffer
            // TODO [divisors]: fix aux segments
            let main_state = main_frame.current();
            let aux_state = aux_frame.current();
            self.boundary_constraints.evaluate_all(
                main_state,
                aux_state,
                x,
                step,
                &mut evaluations[main_evals.len()..],
            );

            // record the result in the evaluation table
            fragment.update_row(i, &evaluations);

            // update x to the next value
            x *= g;
        }
    }

    // TRANSITION CONSTRAINT EVALUATORS
    // --------------------------------------------------------------------------------------------

    /// Evaluates transition constraints of the main execution trace at the specified step of the
    /// constraint evaluation domain.
    ///
    /// `x` is the corresponding domain value at the specified step. That is, x = s * g^step,
    /// where g is the generator of the constraint evaluation domain, and s is the domain offset.
    #[rustfmt::skip]
    fn evaluate_main_transition(
        &self,
        main_frame: &EvaluationFrame<E::BaseField>,
        x: E::BaseField,
        step: usize,
        evaluations: &mut [E::BaseField],
    ) -> Vec<E>{
        // TODO: use a more efficient way to zero out memory
        evaluations.fill(E::BaseField::ZERO);

        // get periodic values at the evaluation step
        let periodic_values = self.periodic_values.get_row(step);

        // evaluate transition constraints over the main segment of the execution trace and save
        // the results into evaluations buffer
        self.air.evaluate_transition(main_frame, periodic_values, evaluations);


        let mut adjustments = BTreeMap::new();
        for group in self.transition_constraints.main_constraints().iter().chain(self.transition_constraints.aux_constraints().iter()) {
            for constraint in group.constraint_information().iter() {
                if !adjustments.contains_key(&constraint.1) {
                    adjustments.insert(constraint.1, x.exp((constraint.1).into()));
                }
            }
        }
        
        // merge transition constraint evaluations into a vector of values based on their divisor;
        // DUMMY COMMENT
        self.transition_constraints.main_constraints().iter().map(|group| 
            group.merge_evaluations2(evaluations, x, &adjustments)
        )
        // self.transition_constraints.main_constraints().iter().map(|group| 
        //     group.merge_evaluations(evaluations, x)
        // )
        .collect::<Vec<_>>()
    }

    /// Evaluates all transition constraints (i.e., for main and auxiliary trace segments) at the
    /// specified step of the constraint evaluation domain.
    ///
    /// `x` is the corresponding domain value at the specified step. That is, x = s * g^step,
    /// where g is the generator of the constraint evaluation domain, and s is the domain offset.
    #[rustfmt::skip]
    fn evaluate_aux_transition(
        &self,
        main_frame: &EvaluationFrame<E::BaseField>,
        aux_frame: &EvaluationFrame<E>,
        x: E::BaseField,
        step: usize,
        evaluations: &mut [E],
    ) -> Vec<E>{
        // TODO: use a more efficient way to zero out memory
        evaluations.fill(E::ZERO);

        // get periodic values at the evaluation step
        let periodic_values = self.periodic_values.get_row(step);

        // evaluate transition constraints over auxiliary trace segments and save the results into
        // evaluations buffer
        self.air.evaluate_aux_transition(
            main_frame,
            aux_frame,
            periodic_values,
            &self.aux_rand_elements,
            evaluations,
        );

        // merge transition constraint evaluations into a vector of values based on their divisor;
        self.transition_constraints.aux_constraints().iter().map(|group| 
            group.merge_evaluations::<E::BaseField, E>(evaluations, x)
        )
        .collect::<Vec<_>>()
    }

    // ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of transition constraints applied against the main segment of the
    /// execution trace.
    fn num_main_transition_constraints(&self) -> usize {
        self.transition_constraints.num_main_constraints()
    }

    /// Returns the number of transition constraints applied against all auxiliary trace segments.
    fn num_aux_transition_constraints(&self) -> usize {
        self.transition_constraints.num_aux_constraints()
    }
}
