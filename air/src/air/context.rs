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
    pub(super) ce_blowup_factor: usize,
    pub(super) trace_domain_generator: B,
    pub(super) lde_domain_generator: B,
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
    /// Panics if `transition_constraint_degrees` is an empty vector.
    pub fn new(
        trace_info: TraceInfo,
        transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        options: ProofOptions,
    ) -> Self {
        Self::new_multi_segment(
            trace_info,
            transition_constraint_degrees,
            Vec::new(),
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
    /// Panics if `transition_constraint_degrees` is an empty vector.
    pub fn new_multi_segment(
        trace_info: TraceInfo,
        main_transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        aux_transition_constraint_degrees: Vec<TransitionConstraintDegree>,
        options: ProofOptions,
    ) -> Self {
        assert!(
            !main_transition_constraint_degrees.is_empty(),
            "at least one transition constraint degree must be specified"
        );

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

        AirContext {
            options,
            trace_info,
            main_transition_constraint_degrees,
            aux_transition_constraint_degrees,
            ce_blowup_factor,
            trace_domain_generator: B::get_root_of_unity(log2(trace_length)),
            lde_domain_generator: B::get_root_of_unity(log2(lde_domain_size)),
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
}
