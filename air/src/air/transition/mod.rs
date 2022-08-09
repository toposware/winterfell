// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{AirContext, BTreeMap, ConstraintDivisor, ExtensionOf, FieldElement, Vec};

mod frame;
pub use frame::EvaluationFrame;

mod degree;
pub use degree::TransitionConstraintDegree;

// CONSTANTS
// ================================================================================================

const MIN_CYCLE_LENGTH: usize = 2;

// TRANSITION CONSTRAINT INFO
// ================================================================================================
/// Metadata for transition constraints of a computation.
///
/// This metadata includes:
/// - List of transition constraint degrees for the main trace segment, as well as for auxiliary
///   trace segments (if any).
/// - Groupings of constraints by their degree, separately for the main trace segment and for
///   auxiliary tace segment.
/// - Index of the divisor used by each constraint.
/// - Divisor of transition constraints for a computation.
pub struct TransitionConstraints<E: FieldElement> {
    main_constraints: Vec<TransitionConstraintGroup<E>>,
    main_constraint_degrees: Vec<TransitionConstraintDegree>,
    main_constraint_divisors: Vec<usize>,
    aux_constraints: Vec<TransitionConstraintGroup<E>>,
    aux_constraint_degrees: Vec<TransitionConstraintDegree>,
    aux_constraint_divisors: Vec<usize>,
    divisors: Vec<ConstraintDivisor<E::BaseField>>,
}

impl<E: FieldElement> TransitionConstraints<E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new instance of [TransitionConstraints] for a computation described by the
    /// specified AIR context.
    ///
    /// # Panics
    /// Panics if the number of transition constraints in the context does not match the number of
    /// provided composition coefficients.
    pub fn new(context: &AirContext<E::BaseField>, composition_coefficients: &[(E, E)]) -> Self {
        assert_eq!(
            context.num_transition_constraints(),
            composition_coefficients.len(),
            "number of transition constraints must match the number of composition coefficient tuples"
        );

        let divisors: Vec<ConstraintDivisor<E::BaseField>> = context
            .divisors
            .iter()
            .map(|divisor| ConstraintDivisor::from_transition::<E>(divisor, context.trace_len()))
            .collect();

        // group constraints by their degree and divisors, separately for constraints against main and auxiliary
        // trace segments

        let (main_constraint_coefficients, aux_constraint_coefficients) =
            composition_coefficients.split_at(context.main_transition_constraint_degrees.len());

        let main_constraint_degrees = context.main_transition_constraint_degrees.clone();
        let main_constraint_divisors = context.main_transition_constraint_divisors.clone();
        let main_constraints = group_constraints(
            &main_constraint_degrees,
            &main_constraint_divisors,
            context,
            main_constraint_coefficients,
            divisors.iter().map(|d| d.degree()).collect(),
        );

        let aux_constraint_degrees = context.aux_transition_constraint_degrees.clone();
        let aux_constraint_divisors = context.aux_transition_constraint_divisors.clone();
        let aux_constraints = group_constraints(
            &aux_constraint_degrees,
            &aux_constraint_divisors,
            context,
            aux_constraint_coefficients,
            divisors.iter().map(|d| d.degree()).collect(),
        );

        Self {
            main_constraints,
            main_constraint_degrees,
            main_constraint_divisors,
            aux_constraints,
            aux_constraint_degrees,
            aux_constraint_divisors,
            divisors,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns transition constraint info for constraints applied against the main trace segment
    /// of a computation grouped by constraint degree.
    pub fn main_constraints(&self) -> &[TransitionConstraintGroup<E>] {
        &self.main_constraints
    }

    /// Returns a list of transition constraint degree descriptors for the main trace segment of
    /// a computation.
    ///
    /// This list will be identical to the list passed into the [AirContext::new()] method as
    /// the `transition_constraint_degrees` parameter, or into [AirContext::new_multi_segment()]
    /// as the `main_transition_constraint_degrees` parameter.
    pub fn main_constraint_degrees(&self) -> &[TransitionConstraintDegree] {
        &self.main_constraint_degrees
    }

    /// Returns the vector of divisor indices corresponding to each main constraint.
    pub fn main_constraints_divisors(&self) -> &[usize] {
        &self.main_constraint_divisors
    }

    /// Returns the number of constraints applied against the main trace segment of a computation.
    pub fn num_main_constraints(&self) -> usize {
        self.main_constraint_degrees.len()
    }

    /// Returns transition constraint info for constraints applied against auxiliary trace segments
    /// of a computation grouped by constraint degree.
    pub fn aux_constraints(&self) -> &[TransitionConstraintGroup<E>] {
        &self.aux_constraints
    }

    /// Returns a list of transition constraint degree descriptors for auxiliary trace segments of
    /// a computation.
    ///
    /// This list will be identical to the list passed into [AirContext::new_multi_segment()]
    /// as the `aux_transition_constraint_degrees` parameter.
    pub fn aux_constraint_degrees(&self) -> &[TransitionConstraintDegree] {
        &self.aux_constraint_degrees
    }

    /// Returns the vector of divisor indices corresponding to each auxiliary constraint.
    pub fn aux_constraints_divisors(&self) -> &[usize] {
        &self.aux_constraint_divisors
    }

    /// Returns the number of constraints applied against auxiliary trace segments of a
    /// computation.
    pub fn num_aux_constraints(&self) -> usize {
        self.aux_constraint_degrees.len()
    }

    /// Returns the list of available divisors for transition constraints.
    pub fn divisors(&self) -> &Vec<ConstraintDivisor<E::BaseField>> {
        &self.divisors
    }

    // CONSTRAINT COMPOSITION
    // --------------------------------------------------------------------------------------------

    /// Computes a linear combination of all transition constraint evaluations and divides the
    /// result by transition constraint divisor.
    ///
    /// A transition constraint is described by a rational function of the form $\frac{C(x)}{z(x)}$,
    /// where:
    /// * $C(x)$ is the constraint polynomial.
    /// * $z(x)$ is the constraint divisor polynomial.
    ///
    /// Thus, this function computes a linear combination of $C(x)$ evaluations. For more detail on
    ///  how this linear combination is computed refer to [TransitionConstraintGroup::merge_evaluations].
    ///
    /// For each constraint group the divisor is decomposed so to combine all constraints we have to
    /// divide with the zero polynomial of the trace domain $x^n-1$ in the end. This minimizes the number
    /// of inversions at the cost of field multiplications.
    pub fn combine_evaluations<F>(
        &self,
        main_evaluations: &[F],
        aux_evaluations: &[E],
        x: F,
        trace_length: usize,
    ) -> E
    where
        F: FieldElement<BaseField = E::BaseField>,
        E: ExtensionOf<F>,
    {
        // merge constraint evaluations for the main trace segment
        let mut result = self.main_constraints().iter().fold(E::ZERO, |acc, group| {
            let custom_divisor = self.divisors()[group.divisor_index].clone();

            // we take the constraint evaluation, ommitting the divisors along with the correction needed to
            // evaluate the divisor. If a group has a divisor that decomposes as D(x)=(X^n-1)/D'(X)
            // where n is the trace length, we multiply the evaluation of the whole group with D'(x).
            // Note that we do this multiplication once for each group.
            // When we have evaluated all groups, we multiply the combined result with
            // (x^n-1).
            let (evaluation, divisor_correction) =
                group.merge_evaluations::<F, F>(main_evaluations, trace_length, x, custom_divisor);
            acc + evaluation * E::from(divisor_correction)
        });

        // merge constraint evaluations for auxiliary trace segments (if any)
        if self.num_aux_constraints() > 0 {
            result += self.aux_constraints().iter().fold(E::ZERO, |acc, group| {
                let custom_divisor = self.divisors()[group.divisor_index].clone();

                let (evaluation, divisor_correction) = group.merge_evaluations::<F, E>(
                    aux_evaluations,
                    trace_length,
                    x,
                    custom_divisor,
                );
                acc + evaluation * E::from(divisor_correction)
            });
        }

        // divide out the evaluation of divisor at x and return the result
        let z = E::from(ConstraintDivisor::evaluate_default_numerator(
            trace_length,
            x,
        ));
        result / z
    }
}

// TRANSITION CONSTRAINT GROUP
// ================================================================================================
/// A group of transition constraints all having the same degree and the same divisor.
///
/// A transition constraint group does not actually store transition constraints - it stores only
/// their indexes and the info needed to compute their random linear combination. The indexes are
/// assumed to be consistent with the order in which constraint evaluations are written into the
/// `evaluation` table by the [Air::evaluate_transition()](crate::Air::evaluate_transition) or
/// [Air::evaluate_aux_transition()](crate::Air::evaluate_aux_transition) function.
#[derive(Clone, Debug)]
pub struct TransitionConstraintGroup<E: FieldElement> {
    degree: TransitionConstraintDegree,
    degree_adjustment: u32,
    indexes: Vec<usize>,
    coefficients: Vec<(E, E)>,
    divisor_index: usize,
}

impl<E: FieldElement> TransitionConstraintGroup<E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new transition constraint group to hold constraints of the specified degree.
    pub(super) fn new(
        degree: TransitionConstraintDegree,
        trace_length: usize,
        composition_degree: usize,
        divisor_degree: usize,
        divisor_index: usize,
    ) -> Self {
        // We want to make sure that once we divide a constraint polynomial by its divisor, the
        // degree of the resulting polynomial will be exactly equal to the composition_degree.
        let target_degree = composition_degree + divisor_degree;
        let evaluation_degree = degree.get_evaluation_degree(trace_length);
        let degree_adjustment = (target_degree - evaluation_degree) as u32;
        TransitionConstraintGroup {
            degree,
            degree_adjustment,
            indexes: vec![],
            coefficients: vec![],
            divisor_index,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns indexes of all constraints in this group.
    pub fn indexes(&self) -> &[usize] {
        &self.indexes
    }

    /// Returns degree descriptors for all constraints in this group.
    pub fn degree(&self) -> &TransitionConstraintDegree {
        &self.degree
    }

    /// Returns divisor index of the group.
    pub fn divisor_index(&self) -> usize {
        self.divisor_index
    }

    /// Adds a new constraint to the group. The constraint is identified by an index in the
    /// evaluation table.
    pub fn add(&mut self, constraint_idx: usize, coefficients: (E, E)) {
        self.indexes.push(constraint_idx);
        self.coefficients.push(coefficients);
    }

    // EVALUATOR
    // --------------------------------------------------------------------------------------------
    /// Computes a linear combination of evaluations relevant to this constraint group.
    ///
    /// The linear combination is computed as follows:
    /// $$
    /// \sum_{i=0}^{k-1}{C_i(x) \cdot D'_i(x)\cdot (\alpha_i + \beta_i \cdot x^d)}
    /// $$
    /// where:
    /// * $C_i(x)$ is the evaluation of the $i$th constraint at `x` (same as `evaluations[i]`).
    /// * $D'_i(x)$ is the divisor correction. Concretely, it is a polynomial such that
    ///   $D_i(x)=(x^n-1)/D_i'(x)$.
    /// * $\alpha$ and $\beta$ are random field elements. In the interactive version of the
    ///   protocol, these are provided by the verifier.
    /// * $d$ is the degree adjustment factor computed as $D + (n - 1) - deg(C_i(x))$, where
    ///   $D$ is the degree of the composition polynomial, $n$ is the length of the execution
    ///   trace, and $deg(C_i(x))$ is the evaluation degree of the $i$th constraint.
    ///
    /// There are two things to note here. First, the degree adjustment factor $d$ is the same
    /// for all constraints in the group (since all constraints have the same degree). Second,
    /// the merged evaluations represent a polynomial of degree $D + n$, which is higher
    /// then the target degree of the composition polynomial. This is because at this stage,
    /// we are merging only the numerators of transition constraints, and we will need to divide
    /// them by the vanishing polynomial of the trace domain. Thus, once we divide out this polynomial
    /// the evaluations will represent a polynomial of degree $D$.
    pub fn merge_evaluations<B, F>(
        &self,
        evaluations: &[F],
        trace_length: usize,
        x: B,
        custom_divisor: ConstraintDivisor<E::BaseField>,
    ) -> (E, B)
    where
        B: FieldElement,
        F: FieldElement<BaseField = B::BaseField> + ExtensionOf<B>,
        E: FieldElement<BaseField = B::BaseField> + ExtensionOf<B> + ExtensionOf<F>,
    {
        // compute degree adjustment factor for this group
        let xp = x.exp(self.degree_adjustment.into());

        // compute linear combination of evaluations as D(x) * (cc_0 + cc_1 * x^p), where D(x)
        // is an evaluation of a particular constraint, and x^p is the degree adjustment factor.
        // also compute the divisor correction needed to combine the constraint with other
        // constraints by dividing out x^n-1
        let mut result = E::ZERO;
        for (&constraint_idx, coefficients) in self.indexes.iter().zip(self.coefficients.iter()) {
            let evaluation = evaluations[constraint_idx];
            result += (coefficients.0 + coefficients.1.mul_base(xp)).mul_base(evaluation);
        }

        let divisor_correction = custom_divisor.evaluate_exemptions_at(x)
            * custom_divisor.evaluate_decomposition(trace_length, x);
        (result, divisor_correction)
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Groups transition constraints by their degree and divisors.
fn group_constraints<E: FieldElement>(
    degrees: &[TransitionConstraintDegree],
    divisors_indices: &[usize],
    context: &AirContext<E::BaseField>,
    coefficients: &[(E, E)],
    divisor_degrees: Vec<usize>,
) -> Vec<TransitionConstraintGroup<E>> {
    // iterate over transition constraint degrees, and assign each constraint to the appropriate
    // group based on its degree and divisor
    let mut groups = BTreeMap::new();
    for (i, degree) in degrees.iter().enumerate() {
        let evaluation_degree = degree.get_evaluation_degree(context.trace_len());
        let group = groups
            // The tree key contains the degree and the divisor index
            .entry((evaluation_degree, divisors_indices[i]))
            .or_insert_with(|| {
                TransitionConstraintGroup::new(
                    degree.clone(),
                    context.trace_len(),
                    context.composition_degree(),
                    divisor_degrees[divisors_indices[i]],
                    divisors_indices[i],
                )
            });
        group.add(i, coefficients[i]);
    }

    // convert from hash map into a vector and return
    groups.into_iter().map(|e| e.1).collect()
}
