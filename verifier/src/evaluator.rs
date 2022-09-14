// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use air::{Air, AuxTraceRandElements, ConstraintCompositionCoefficients, EvaluationFrame};
use math::{polynom, FieldElement};
use utils::collections::Vec;
use utils::collections::{BTreeMap, BTreeSet};

// CONSANTS
// ================================================================================================

// Defines the cost of an inversion in terms of multiplications.
const INVERSION_COST: usize = 50;

// ================================================================================================

// CONSTRAINT EVALUATION
// ================================================================================================

/// Evaluates constraints for the specified evaluation frame.
pub fn evaluate_constraints<A: Air, E: FieldElement<BaseField = A::BaseField>>(
    air: &A,
    composition_coefficients: ConstraintCompositionCoefficients<E>,
    main_trace_frame: &EvaluationFrame<E>,
    aux_trace_frame: &Option<EvaluationFrame<E>>,
    aux_rand_elements: AuxTraceRandElements<E>,
    x: E,
) -> E {
    // 1 ----- evaluate transition constraints ----------------------------------------------------

    // initialize a buffer to hold transition constraint evaluations
    let t_constraints = air.get_transition_constraints(&composition_coefficients.transition);

    // compute values of periodic columns at x
    let periodic_values = air
        .get_periodic_column_polys()
        .iter()
        .map(|poly| {
            let num_cycles = air.trace_length() / poly.len();
            let x = x.exp((num_cycles as u32).into());
            polynom::eval(poly, x)
        })
        .collect::<Vec<_>>();

    // dictionary to chache exponentiations for x
    let mut xps: BTreeMap<usize, E> = BTreeMap::new();
    // dictionary to cache exponentiations for offsets
    let mut gxps: BTreeMap<usize, E::BaseField> = BTreeMap::new();
    let custom_divisor_values = air
        .get_custom_divisors()
        .iter()
        .map(|(period, offsets)| {
            // We can evaluate the divisor in two ways:
            //      1. compute X^n-1 / \Prod X^k - offset.
            //         This involves offsets.len() multiplication and one iversion per point
            //      2. compute \Prod X^k - c_offset where c_offset are elements not
            //         included in the offset
            //         This involves period - offsets.len() multiplication
            if offsets.len() + INVERSION_COST < period - offsets.len() {
                // compute X^n - 1
                let xp: E = *xps
                    .entry(air.trace_length())
                    .or_insert_with(|| x.exp(((air.trace_length()) as u64).into()));
                let numerator = xp - E::ONE;

                // compute \Prod X^{n/p - g^offset}
                let mut denominator = E::ONE;
                for offset in offsets {
                    let g_offset_dlog = air.trace_length() / period * offset;
                    let g_offset = *gxps.entry(g_offset_dlog).or_insert_with(|| {
                        air.trace_domain_generator()
                            .exp((g_offset_dlog as u64).into())
                    });
                    let xp = *xps
                        .entry(air.trace_length() / period)
                        .or_insert_with(|| x.exp(((air.trace_length() / period) as u64).into()));
                    denominator *= xp - g_offset.into();
                }
                numerator / denominator
            } else {
                // compute the complementary offsets {0..period} \ offsets
                let mut c_offsets = BTreeSet::new();
                for i in 0..*period {
                    c_offsets.insert(i);
                }
                for offset in offsets {
                    c_offsets.remove(offset);
                }

                let mut evaluation = E::ONE;
                for offset in c_offsets {
                    let g_offset_dlog = air.trace_length() / period * offset;
                    let g_offset = *gxps.entry(g_offset_dlog).or_insert_with(|| {
                        air.trace_domain_generator()
                            .exp((g_offset_dlog as u64).into())
                    });
                    let xp = *xps
                        .entry(air.trace_length() / period)
                        .or_insert_with(|| x.exp(((air.trace_length() / period) as u64).into()));

                    evaluation *= xp - g_offset.into();
                }
                evaluation
            }
        })
        .collect::<Vec<_>>();

    // evaluate transition constraints for the main trace segment
    let mut t_evaluations1 = E::zeroed_vector(t_constraints.num_main_constraints());

    let periodic_values = [periodic_values, custom_divisor_values].concat();

    air.evaluate_transition(main_trace_frame, &periodic_values, &mut t_evaluations1);

    // evaluate transition constraints for auxiliary trace segments (if any)
    let mut t_evaluations2 = E::zeroed_vector(t_constraints.num_aux_constraints());
    if let Some(aux_trace_frame) = aux_trace_frame {
        air.evaluate_aux_transition(
            main_trace_frame,
            aux_trace_frame,
            &periodic_values,
            &aux_rand_elements,
            &mut t_evaluations2,
        );
    }

    // merge all constraint evaluations into a single value by computing their random linear
    // combination using coefficients drawn from the public coin. this also divides the result
    // by the divisor of transition constraints.
    let mut result = t_constraints.combine_evaluations::<E>(&t_evaluations1, &t_evaluations2, x);

    // 2 ----- evaluate boundary constraints ------------------------------------------------------

    // get boundary constraints grouped by common divisor from the AIR
    let b_constraints =
        air.get_boundary_constraints(&aux_rand_elements, &composition_coefficients.boundary);

    // cache power of x here so that we only re-compute it when degree_adjustment changes
    let mut degree_adjustment = b_constraints.main_constraints()[0].degree_adjustment();
    let mut xp = x.exp(degree_adjustment.into());

    // iterate over boundary constraint groups for the main trace segment (each group has a
    // distinct divisor), evaluate constraints in each group and add their combination to the
    // result
    for group in b_constraints.main_constraints().iter() {
        // if adjustment degree hasn't changed, no need to recompute `xp` - so just reuse the
        // previous value; otherwise, compute new `xp`
        if group.degree_adjustment() != degree_adjustment {
            degree_adjustment = group.degree_adjustment();
            xp = x.exp(degree_adjustment.into());
        }
        // evaluate all constraints in the group, and add the evaluation to the result
        result += group.evaluate_at(main_trace_frame.current(), x, xp);
    }

    // iterate over boundary constraint groups for auxiliary trace segments (each group has a
    // distinct divisor), evaluate constraints in each group and add their combination to the
    // result
    if let Some(aux_trace_frame) = aux_trace_frame {
        for group in b_constraints.aux_constraints().iter() {
            // if adjustment degree hasn't changed, no need to recompute `xp` - so just reuse the
            // previous value; otherwise, compute new `xp`
            if group.degree_adjustment() != degree_adjustment {
                degree_adjustment = group.degree_adjustment();
                xp = x.exp(degree_adjustment.into());
            }
            // evaluate all constraints in the group, and add the evaluation to the result
            result += group.evaluate_at(aux_trace_frame.current(), x, xp);
        }
    }

    result
}
