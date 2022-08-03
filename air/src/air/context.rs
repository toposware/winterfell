// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{air::TransitionConstraintDegree, ProofOptions, TraceInfo};
use math::{log2, StarkField};
use utils::collections::Vec;

// AIR CONTEXT
// ================================================================================================
/// STARK parameters and trace properties for a specific execution of a computation.
#[derive(Clone, PartialEq, Eq)]
pub struct AirContext<B: StarkField> {
    pub(super) options: ProofOptions,
    pub(super) trace_info: TraceInfo,
    pub(super) main_transition_constraint_degrees: Vec<TransitionConstraintDegree>,
    pub(super) aux_transition_constraint_degrees: Vec<TransitionConstraintDegree>,

    // Divisor indices to use for main and transition constraint
    pub(super) main_transition_constraint_divisors: Vec<usize>,
    pub(super) aux_transition_constraint_divisors: Vec<usize>,

    pub(super) num_main_assertions: usize,
    pub(super) num_aux_assertions: usize,
    pub(super) ce_blowup_factor: usize,
    pub(super) trace_domain_generator: B,
    pub(super) lde_domain_generator: B,
    // This defines the default divisor which is all points in trace except the num_transition_exemptions last ones (by default 1).
    // Each custom divisor is a divisor of the default divisor.
    //
    // TODO: [divisors] Remove this. Each divisor should be indepenendent and expressed as a polynomial D(X) = Z(X)/D'(X).
    // Z(X) is the polynomial that vanishes on the trace domain and D(X). This will allow to later divide all constraints once using
    // Z(X) instead of dividing each constraint with its divisor.
    pub(super) num_transition_exemptions: usize,
    // The vector of available divisors for the given AIR.
    pub(super) divisors: Vec<usize>,
}

impl<B: StarkField> AirContext<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------
    /// Returns a new instance of [AirContext] instantiated for computations which require a single
    /// execution trace segment.
    ///
    /// The list of transition constraint degrees defines the total number of transition
    /// constraints and their expected degrees. Constraint evaluations computed by
    /// [Air::evaluate_transition()](crate::Air::evaluate_transition) function are expected to be
    /// in the order defined by this list.
    ///
    /// # Panics
    /// Panics if
    /// * `transition_constraint_degrees` is an empty vector.
    /// * `num_assertions` is zero.
    /// * Blowup factor specified by the provided `options` is too small to accommodate degrees
    ///   of the specified transition constraints.
    /// * `trace_info` describes a multi-segment execution trace.
    pub fn new(
        trace_info: TraceInfo,
        transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        num_assertions: usize,
        options: ProofOptions,
    ) -> Self {
        assert!(
            !trace_info.is_multi_segment(),
            "provided trace info describes a multi-segment execution trace"
        );
        Self::new_multi_segment(
            trace_info,
            transition_constraint_degrees,
            Vec::new(),
            num_assertions,
            0,
            options,
        )
    }

    /// Returns a new instance of [AirContext] instantiated for computations which require multiple
    /// execution trace segments.
    ///
    /// The lists of transition constraint degrees defines the total number of transition
    /// constraints and their expected degrees. Constraint evaluations computed by
    /// [Air::evaluate_transition()](crate::Air::evaluate_transition) function are expected to be
    /// in the order defined by `main_transition_constraint_degrees` list. Constraint evaluations
    /// computed by [Air::evaluate_aux_transition()](crate::Air::evaluate_aux_transition) function
    /// are expected to be in the order defined by `aux_transition_constraint_degrees` list.
    ///
    /// # Panics
    /// Panics if
    /// * `main_transition_constraint_degrees` is an empty vector.
    /// * `num_main_assertions` is zero.
    /// * `trace_info.is_multi_segment() == true` but:
    ///   - `aux_transition_constraint_degrees` is an empty vector.
    ///   - `num_aux_assertions` is zero.
    /// * `trace_info.is_multi_segment() == false` but:
    ///   - `aux_transition_constraint_degrees` is a non-empty vector.
    ///   - `num_aux_assertions` is greater than zero.
    /// * Blowup factor specified by the provided `options` is too small to accommodate degrees
    ///   of the specified transition constraints.
    pub fn new_multi_segment(
        trace_info: TraceInfo,
        main_transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        aux_transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        num_main_assertions: usize,
        num_aux_assertions: usize,
        options: ProofOptions,
    ) -> Self {
        assert!(
            !main_transition_constraint_degrees.is_empty(),
            "at least one transition constraint degree must be specified"
        );
        assert!(
            num_main_assertions > 0,
            "at least one assertion must be specified"
        );

        if trace_info.is_multi_segment() {
            assert!(
                !aux_transition_constraint_degrees.is_empty(),
                "at least one transition constraint degree must be specified for auxiliary trace segments"
            );
            assert!(
                num_aux_assertions > 0,
                "at least one assertion must be specified against auxiliary trace segments"
            );
        } else {
            assert!(
                aux_transition_constraint_degrees.is_empty(),
                "auxiliary transition constraint degrees specified for a single-segment trace"
            );
            assert!(
                num_aux_assertions == 0,
                "auxiliary assertions specified for a single-segment trace"
            );
        }

        // determine minimum blowup factor needed to evaluate transition constraints by taking
        // the blowup factor of the highest degree constraint
        let mut ce_blowup_factor = 0;
        for degree in main_transition_constraint_degrees.iter() {
            if degree.min_blowup_factor() > ce_blowup_factor {
                ce_blowup_factor = degree.min_blowup_factor();
            }
        }

        for degree in aux_transition_constraint_degrees.iter() {
            if degree.min_blowup_factor() > ce_blowup_factor {
                ce_blowup_factor = degree.min_blowup_factor();
            }
        }

        assert!(
            options.blowup_factor() >= ce_blowup_factor,
            "blowup factor too small; expected at least {}, but was {}",
            ce_blowup_factor,
            options.blowup_factor()
        );

        let trace_length = trace_info.length();
        let lde_domain_size = trace_length * options.blowup_factor();

        // The default AIR uses the standard divisor (all transitions in trace except the last one).
        // Mutate divisors, main_transition_constraints and aux_transition_constraints to modify this behavior
        AirContext {
            options,
            trace_info,
            main_transition_constraint_divisors: vec![0; main_transition_constraint_degrees.len()],
            aux_transition_constraint_divisors: vec![0; aux_transition_constraint_degrees.len()],
            main_transition_constraint_degrees,
            aux_transition_constraint_degrees,
            num_main_assertions,
            num_aux_assertions,
            ce_blowup_factor,
            trace_domain_generator: B::get_root_of_unity(log2(trace_length)),
            lde_domain_generator: B::get_root_of_unity(log2(lde_domain_size)),
            num_transition_exemptions: 1,
            // The default divisor
            divisors: vec![1],
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns length of the execution trace for an instance of a computation.
    ///
    // This is guaranteed to be a power of two greater than or equal to 8.
    pub fn trace_len(&self) -> usize {
        self.trace_info.length()
    }

    /// Returns degree of trace polynomials for an instance of a computation.
    ///
    /// The degree is always `trace_length` - 1.
    pub fn trace_poly_degree(&self) -> usize {
        self.trace_info.length() - 1
    }

    /// Returns size of the constraint evaluation domain.
    ///
    /// This is guaranteed to be a power of two, and is equal to `trace_length * ce_blowup_factor`.
    pub fn ce_domain_size(&self) -> usize {
        self.trace_info.length() * self.ce_blowup_factor
    }

    /// Returns the degree to which all constraint polynomials are normalized before they are
    /// composed together.
    ///
    /// This degree is always `ce_domain_size` - 1.
    pub fn composition_degree(&self) -> usize {
        self.ce_domain_size() - 1
    }

    /// Returns the size of the low-degree extension domain.
    ///
    /// This is guaranteed to be a power of two, and is equal to `trace_length * lde_blowup_factor`.
    pub fn lde_domain_size(&self) -> usize {
        self.trace_info.length() * self.options.blowup_factor()
    }

    /// Returns the number of transition constraints for a computation.
    ///
    /// The number of transition constraints is defined by the total number of transition
    /// constraint degree descriptors (for both the main and the auxiliary trace constraints).
    /// This number is used to determine how many transition constraint coefficients need to be
    /// generated for merging transition constraints into a composition polynomial.
    pub fn num_transition_constraints(&self) -> usize {
        self.main_transition_constraint_degrees.len() + self.aux_transition_constraint_degrees.len()
    }

    /// Returns the number of transition constraints placed against the main trace segment.
    pub fn num_main_transition_constraints(&self) -> usize {
        self.main_transition_constraint_degrees.len()
    }

    /// Returns the number of transition constraints placed against all auxiliary trace segments.
    pub fn num_aux_transition_constraints(&self) -> usize {
        self.aux_transition_constraint_degrees.len()
    }

    /// Returns the total number of assertions defined for a computation.
    ///
    /// The number of assertions consists of the assertions placed against the main segment of an
    /// execution trace as well as assertions placed against all auxiliary trace segments.
    pub fn num_assertions(&self) -> usize {
        self.num_main_assertions + self.num_aux_assertions
    }

    /// Returns the number of rows at the end of an execution trace to which transition constraints
    /// do not apply.
    ///
    /// This is guaranteed to be at least 1 (which is the default value), but could be greater.
    /// The maximum number of exemptions is determined by a combination of transition constraint
    /// degrees and blowup factor specified for the computation.
    // TODO: [divisors] should probably not be needed
    pub fn num_transition_exemptions(&self) -> usize {
        self.num_transition_exemptions
    }

    /// Returns a reference to the indices of the divisor used by the AIR main constraints
    pub fn main_transition_constraint_divisors(&self) -> &[usize] {
        &self.main_transition_constraint_divisors
    }

    /// Returns a reference to the indices of the divisor used by the AIR aux constraints
    pub fn aux_transition_constraint_divisors(&self) -> &[usize] {
        &self.aux_transition_constraint_divisors
    }

    /// Returns a reference to the available divisors used by the AIR
    pub fn divisors(&self) -> &[usize] {
        &self.divisors
    }
    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Sets the number of transition exemptions for this context.
    ///
    /// # Panics
    /// Panics if:
    /// * The number of exemptions is zero.
    /// * The number of exemptions exceeds half of the trace length.
    /// * Given the combination of transition constraints degrees and the blowup factor in this
    ///   context, the number of exemptions is too larger for a valid computation of the constraint
    ///   composition polynomial.
    pub fn set_num_transition_exemptions(mut self, n: usize) -> Self {
        assert!(
            n > 0,
            "number of transition exemptions must be greater than zero"
        );
        // exemptions which are for more than half the trace plus one are probably a mistake
        assert!(
            n <= self.trace_len() / 2 + 1,
            "number of transition exemptions cannot exceed {}, but was {}",
            self.trace_len() / 2 + 1,
            n
        );
        // make sure the composition polynomial can be computed correctly with the specified
        // number of exemptions
        for degree in self
            .main_transition_constraint_degrees
            .iter()
            .chain(self.aux_transition_constraint_degrees.iter())
        {
            let eval_degree = degree.get_evaluation_degree(self.trace_len());
            let max_exemptions = self.composition_degree() + self.trace_len() - eval_degree;
            assert!(
                n <= max_exemptions,
                "number of transition exemptions cannot exceed: {}, but was {}",
                max_exemptions,
                n
            )
        }

        self.num_transition_exemptions = n;
        // TODO: [divisors] currently this corresponds to the default divisor, so changing this means we need to change
        // the default divisor as well. Remove completely num_transition_exemptions parameter and only have custom divisors.
        self.divisors[0] = n;
        self
    }

    /// Sets custom divisors for this context.
    ///
    /// # Panics
    /// Panics if:
    /// * The number of divisors is different than the number of constraints.
    /// * A divisor index exceeds the number of available divisors.
    /// * A divisor is invalid (currently checking exemptions number only)
    /// * Given the combination of transition constraints degrees and the blowup factor in this
    ///   context, the number of exemptions is too larger for a valid computation of the constraint
    ///   composition polynomial.
    pub fn set_custom_divisors(
        mut self,
        divisors: Vec<usize>,
        main_constraint_divisors: Vec<usize>,
        aux_constraint_divisors: Vec<usize>,
    ) -> Self {
        // assert the number of divisors coincides with the number of constraints
        assert_eq!(
            main_constraint_divisors.len(),
            self.main_transition_constraint_degrees.len(),
            "number of main custom divisors {} is different than the number of constraints {}",
            main_constraint_divisors.len(),
            self.main_transition_constraint_degrees.len()
        );
        assert_eq!(
            aux_constraint_divisors.len(),
            self.aux_transition_constraint_degrees.len(),
            "number of aux custom divisors {} is different than the number of constraints {}",
            aux_constraint_divisors.len(),
            self.aux_transition_constraint_degrees.len()
        );

        // assert all divisor indexes are valid
        for index in main_constraint_divisors.iter() {
            assert!(
                *index <= divisors.len(),
                "main constraint divisor with index {} does not exist, max divisor index is {}",
                index,
                divisors.len()
            );
        }
        for index in aux_constraint_divisors.iter() {
            assert!(
                *index <= divisors.len(),
                "aux constraint divisor with index {} does not exist, max divisor index is {}",
                index,
                divisors.len()
            );
        }

        // assert all divisors are valid
        // TODO: [divisors] refine this
        for divisor in divisors.iter() {
            assert!(
                *divisor > 0,
                "number of transition exemptions must be greater than zero"
            );
            // exemptions which are for more than half the trace plus one are probably a mistake
            assert!(
                *divisor <= self.trace_len() / 2 + 1,
                "number of transition exemptions cannot exceed {}, but was {}",
                self.trace_len() / 2 + 1,
                divisor
            );
        }

        let min_divisor_degree = *divisors.iter().min().unwrap();
        // make sure the composition polynomial can be computed correctly with the specified
        // number of exemptions
        // TODO: [divisors] refine this
        for degree in self
            .main_transition_constraint_degrees
            .iter()
            .chain(self.aux_transition_constraint_degrees.iter())
        {
            let eval_degree = degree.get_evaluation_degree(self.trace_len());
            let max_exemptions = self.composition_degree() + self.trace_len() - eval_degree;
            assert!(
                min_divisor_degree <= max_exemptions,
                "number of transition exemptions cannot exceed: {}, but was {}",
                max_exemptions,
                min_divisor_degree
            )
        }

        self.main_transition_constraint_divisors = main_constraint_divisors;
        self.aux_transition_constraint_divisors = aux_constraint_divisors;
        self.divisors.extend(divisors);
        self
    }
}
