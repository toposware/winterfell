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
/// - Divisor of transition constraints for a computation along with the indexes of the cosets.
pub struct TransitionConstraints<E: FieldElement> {
    main_constraints: Vec<TransitionConstraintGroup<E>>,
    main_constraint_degrees: Vec<TransitionConstraintDegree>,
    main_constraint_divisors: Vec<usize>,
    aux_constraints: Vec<TransitionConstraintGroup<E>>,
    aux_constraint_degrees: Vec<TransitionConstraintDegree>,
    aux_constraint_divisors: Vec<usize>,
    divisors: Vec<(ConstraintDivisor<E::BaseField>)>,
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

        let divisors: Vec<_> = context
            .divisors
            .iter()
            .map(|divisor| ConstraintDivisor::from_transition2(context.trace_len(), divisor))
            .collect();

        // group constraints by their degreeivisorsivisor, separately for constraints against main and auxiliary
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
            divisors.iter().map(|d| d.0.degree()).collect(),
        );

        let aux_constraint_degrees = context.aux_transition_constraint_degrees.clone();
        let aux_constraint_divisors = context.aux_transition_constraint_divisors.clone();
        let aux_constraints = group_constraints(
            &aux_constraint_degrees,
            &aux_constraint_divisors,
            context,
            aux_constraint_coefficients,
            divisors.iter().map(|d| d.0.degree()).collect(),
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

    /// Returns the number of constraints applied against auxiliary trace segments of a
    /// computation.
    pub fn num_aux_constraints(&self) -> usize {
        self.aux_constraint_degrees.len()
    }

    /// Returns a divisor for transition constraints.
    ///
    /// All transition constraints have the same divisor which has the form:
    /// $$
    /// z(x) = \frac{x^n - 1}{x - g^{n - 1}}
    /// $$
    /// where: $n$ is the length of the execution trace and $g$ is the generator of the trace
    /// domain.
    ///
    /// This divisor specifies that transition constraints must hold on all steps of the
    /// execution trace except for the last one.
    pub fn divisor(&self) -> &ConstraintDivisor<E::BaseField> {
        &self.divisor
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
    /// Since, the divisor polynomial is the same for all transition constraints (see
    /// [ConstraintDivisor::from_transition]), we can divide the linear combination by the
    /// divisor rather than dividing each individual $C(x)$ evaluation. This requires executing only
    /// one division at the end.
    pub fn combine_evaluations<F>(&self, main_evaluations: &[F], aux_evaluations: &[E], x: F) -> E
    where
        F: FieldElement<BaseField = E::BaseField>,
        E: ExtensionOf<F>,
    {
        // merge constraint evaluations for the main trace segment
        let mut result = self.main_constraints().iter().fold(E::ZERO, |acc, group| {
            acc + group.merge_evaluations::<F, F>(main_evaluations, x)
        });

        // merge constraint evaluations for auxiliary trace segments (if any)
        if self.num_aux_constraints() > 0 {
            result += self.aux_constraints().iter().fold(E::ZERO, |acc, group| {
                acc + group.merge_evaluations::<F, E>(aux_evaluations, x)
            });
        }

        // divide out the evaluation of divisor at x and return the result
        let z = E::from(self.divisor.evaluate_at(x));
        result / z
    }
}

// TRANSITION CONSTRAINT GROUP
// ================================================================================================
/// A group of transition constraints all having the same degree.
///
/// A transition constraint group does not actually store transition constraints - it stores only
/// their indexes and the info needed to compute their random linear combination. The indexes are
/// assumed to be consistent with the order in which constraint evaluations are written into the
/// `evaluation` table by the [Air::evaluate_transition()](crate::Air::evaluate_transition) or
/// [Air::evaluate_aux_transition()](crate::Air::evaluate_aux_transition) function.
// TODO [divisors]: DOCS
#[derive(Clone, Debug)]
pub struct TransitionConstraintGroup<E: FieldElement> {
    divisor_index: usize,
    divisor_degree: usize,
    // information regarding contraints inside the group: indices, degree_adjustment, and coefficients
    constraint_information: Vec<(usize, u32, (E, E))>,
}

impl<E: FieldElement> TransitionConstraintGroup<E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new transition constraint group to hold constraints of the specified degree.
    pub(super) fn new(divisor_index: usize, divisor_degree: usize) -> Self {
        TransitionConstraintGroup {
            divisor_index: divisor_index,
            divisor_degree: divisor_degree,
            constraint_information: vec![],
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns divisor index of the group.
    pub fn divisor_index(&self) -> usize {
        self.divisor_index()
    }

    /// Returns divisor index of the group.
    pub fn divisor_degree(&self) -> usize {
        self.divisor_degree()
    }

    /// Returns indexes of all constraints in this group.
    pub fn constraint_information(&self) -> &[(usize, u32, (E, E))] {
        &self.constraint_information
    }

    /// Adds a new constraint to the group. The constraint is identified by an index in the
    /// evaluation table.
    pub fn add(
        &mut self,
        composition_degree: usize,
        trace_length: usize,
        constraint_idx: usize,
        degree: TransitionConstraintDegree,
        coefficients: (E, E),
    ) {
        let target_degree = composition_degree + self.divisor_degree;
        let evaluation_degree = degree.get_evaluation_degree(trace_length);
        let degree_adjustment = (target_degree - evaluation_degree) as u32;
        self.constraint_information
            .push((constraint_idx, degree_adjustment, coefficients));
    }

    // EVALUATOR
    // --------------------------------------------------------------------------------------------
    /// Computes a linear combination of evaluations relevant to this constraint group.
    ///
    /// The linear combination is computed as follows:
    /// $$
    /// \sum_{i=0}^{k-1}{C_i(x) \cdot (\alpha_i + \beta_i \cdot x^d)}
    /// $$
    /// where:
    /// * $C_i(x)$ is the evaluation of the $i$th constraint at `x` (same as `evaluations[i]`).
    /// * $\alpha$ and $\beta$ are random field elements. In the interactive version of the
    ///   protocol, these are provided by the verifier.
    /// * $d$ is the degree adjustment factor computed as $D + (n - 1) - deg(C_i(x))$, where
    ///   $D$ is the degree of the composition polynomial, $n$ is the length of the execution
    ///   trace, and $deg(C_i(x))$ is the evaluation degree of the $i$th constraint.
    ///
    /// There are two things to note here. First, the degree adjustment factor $d$ is the same
    /// for all constraints in the group (since all constraints have the same degree). Second,
    /// the merged evaluations represent a polynomial of degree $D + n - 1$, which is higher
    /// then the target degree of the composition polynomial. This is because at this stage,
    /// we are merging only the numerators of transition constraints, and we will need to divide
    /// them by the divisor later on. The degree of the divisor for transition constraints is
    /// always $n - 1$. Thus, once we divide out the divisor, the evaluations will represent a
    /// polynomial of degree $D$.
    pub fn merge_evaluations<B, F>(&mut self, evaluations: &[F], x: B) -> E
    where
        B: FieldElement,
        F: FieldElement<BaseField = B::BaseField> + ExtensionOf<B>,
        E: FieldElement<BaseField = B::BaseField> + ExtensionOf<B> + ExtensionOf<F>,
    {
        // TODO [divisors]: Precompute degree adjustments to avoid recomputing them across groups

        // compute degree adjustments factors for this group
        let xp = self
            .constraint_information()
            .iter()
            .map(|&constraint_info| x.exp(constraint_info.1.into()))
            .collect::<Vec<_>>();

        // compute linear combination of evaluations as D(x) * (cc_0 + cc_1 * x^p), where D(x)
        // is an evaluation of a particular constraint, and x^p is the degree adjustment factor
        let mut result = E::ZERO;
        for (i, (constraint_idx, _, coefficients)) in
            self.constraint_information().iter().enumerate()
        {
            let evaluation = evaluations[*constraint_idx];
            result += (coefficients.0 + coefficients.1.mul_base(xp[i])).mul_base(evaluation);
        }
        result
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Groups transition constraints by their divisor.
fn group_constraints<E: FieldElement>(
    degrees: &[TransitionConstraintDegree],
    divisors_indices: &[usize],
    context: &AirContext<E::BaseField>,
    coefficients: &[(E, E)],
    divisor_degrees: &[usize],
) -> Vec<TransitionConstraintGroup<E>> {
    // iterate over transition constraint degrees, and assign each constraint to the appropriate
    // group based on its divisor
    let mut groups = BTreeMap::new();
    for (i, (divisor, degree)) in divisors_indices
        .iter()
        .zip(divisor_degrees.iter())
        .enumerate()
    {
        let group = groups
            // The tree key contains the degree and the divisor index
            .entry(divisors_indices[i])
            .or_insert_with(|| {
                TransitionConstraintGroup::new(
                    divisor_degrees[divisors_indices[i]],
                    divisors_indices[i],
                )
            });
        // degree.clone(),
        // context.trace_len(),
        // context.composition_degree(),
        group.add(
            context.composition_degree(),
            context.trace_len(),
            i,
            coefficients[i],
        );
    }

    // convert from hash map into a vector and return
    groups.into_iter().map(|e| e.1).collect()
}
