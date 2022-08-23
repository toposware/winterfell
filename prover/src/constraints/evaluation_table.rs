// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{CompositionPoly, ConstraintDivisor, ProverError, StarkDomain};
use math::{batch_inversion, fft, ExtensionOf, FieldElement, StarkField};
use utils::{
    batch_iter_mut,
    collections::{BTreeMap, Vec},
    iter_mut, uninit_vector,
};

#[cfg(debug_assertions)]
use air::TransitionConstraints;

#[cfg(feature = "concurrent")]
use utils::iterators::*;

// CONSTANTS
// ================================================================================================

const MIN_FRAGMENT_SIZE: usize = 16;

// CONSTRAINT EVALUATION TABLE
// ================================================================================================

pub struct ConstraintEvaluationTable<E: FieldElement> {
    evaluations: Vec<Vec<E>>,
    divisors: Vec<ConstraintDivisor<E::BaseField>>,
    domain_offset: E::BaseField,
    trace_length: usize,

    #[cfg(debug_assertions)]
    main_transition_evaluations: Vec<Vec<E::BaseField>>,
    main_divisor_indices: Vec<usize>,
    #[cfg(debug_assertions)]
    aux_transition_evaluations: Vec<Vec<E>>,
    aux_divisor_indices: Vec<usize>,
    #[cfg(debug_assertions)]
    expected_transition_degrees: Vec<usize>,
}

impl<E: FieldElement> ConstraintEvaluationTable<E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new constraint evaluation table with number of columns equal to the number of
    /// specified divisors, and number of rows equal to the size of constraint evaluation domain.
    #[cfg(not(debug_assertions))]
    pub fn new(
        domain: &StarkDomain<E::BaseField>,
        divisors: Vec<ConstraintDivisor<E::BaseField>>,
    ) -> Self {
        let num_columns = divisors.len();
        let num_rows = domain.ce_domain_size();
        ConstraintEvaluationTable {
            evaluations: uninit_matrix(num_columns, num_rows),
            divisors,
            domain_offset: domain.offset(),
            trace_length: domain.trace_length(),
        }
    }

    /// Similar to the as above constructor but used in debug mode. In debug mode we also want
    /// to keep track of all evaluated transition constraints so that we can verify that their
    /// expected degrees match their actual degrees.
    #[cfg(debug_assertions)]
    pub fn new(
        domain: &StarkDomain<E::BaseField>,
        divisors: Vec<ConstraintDivisor<E::BaseField>>,
        transition_constraints: &TransitionConstraints<E>,
    ) -> Self {
        let num_columns = divisors.len();
        let num_rows = domain.ce_domain_size();
        let num_tm_columns = transition_constraints.num_main_constraints();
        let num_ta_columns = transition_constraints.num_aux_constraints();

        // collect expected degrees for all transition constraints to compare them against actual
        // degrees; we do this in debug mode only because this comparison is expensive
        let expected_transition_degrees =
            build_transition_constraint_degrees(transition_constraints, domain.trace_length());

        ConstraintEvaluationTable {
            evaluations: uninit_matrix(num_columns, num_rows),
            divisors,
            domain_offset: domain.offset(),
            trace_length: domain.trace_length(),
            main_transition_evaluations: uninit_matrix(num_tm_columns, num_rows),
            main_divisor_indices: transition_constraints.main_constraints_divisors().to_vec(),
            aux_transition_evaluations: uninit_matrix(num_ta_columns, num_rows),
            aux_divisor_indices: transition_constraints.aux_constraints_divisors().to_vec(),
            expected_transition_degrees,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of rows in this table. This is the same as the size of the constraint
    /// evaluation domain.
    pub fn num_rows(&self) -> usize {
        self.evaluations[0].len()
    }

    /// Returns number of columns in this table. The first column always contains the value of
    /// combined transition constraint evaluations; the remaining columns contain values of
    /// assertion constraint evaluations combined based on common divisors.
    #[allow(dead_code)]
    pub fn num_columns(&self) -> usize {
        self.evaluations.len()
    }

    /// Returns the list of transition constraint divisors.
    pub fn divisors(&self) -> &[ConstraintDivisor<E::BaseField>] {
        &self.divisors
    }

    // TABLE FRAGMENTS
    // --------------------------------------------------------------------------------------------

    /// Break the table into the number of specified fragments. All fragments can be updated
    /// independently - e.g. in different threads.
    pub fn fragments(&mut self, num_fragments: usize) -> Vec<EvaluationTableFragment<E>> {
        let fragment_size = self.num_rows() / num_fragments;
        assert!(
            fragment_size >= MIN_FRAGMENT_SIZE,
            "fragment size must be at least {}, but was {}",
            MIN_FRAGMENT_SIZE,
            fragment_size
        );

        // break evaluations into fragments
        let evaluation_data = make_fragments(&mut self.evaluations, num_fragments);

        #[cfg(debug_assertions)]
        let result = {
            // in debug mode, also break individual transition evaluations into fragments
            let tm_evaluation_data =
                make_fragments(&mut self.main_transition_evaluations, num_fragments);
            let ta_evaluation_data =
                make_fragments(&mut self.aux_transition_evaluations, num_fragments);

            evaluation_data
                .into_iter()
                .zip(tm_evaluation_data)
                .zip(ta_evaluation_data)
                .enumerate()
                .map(|(i, ((evaluations, tm_evaluations), ta_evaluations))| {
                    EvaluationTableFragment {
                        offset: i * fragment_size,
                        evaluations,
                        tm_evaluations,
                        ta_evaluations,
                    }
                })
                .collect()
        };

        #[cfg(not(debug_assertions))]
        let result = evaluation_data
            .into_iter()
            .enumerate()
            .map(|(i, evaluations)| EvaluationTableFragment {
                offset: i * fragment_size,
                evaluations,
            })
            .collect();

        result
    }

    // CONSTRAINT COMPOSITION
    // --------------------------------------------------------------------------------------------
    /// Divides constraint evaluation columns by their respective divisor (in evaluation form),
    /// combines the results into a single column, and interpolates this column into a composition
    /// polynomial in coefficient form.
    pub fn into_poly(self) -> Result<CompositionPoly<E>, ProverError> {
        let domain_offset = self.domain_offset;

        // allocate memory for the combined polynomial
        let mut combined_poly = E::zeroed_vector(self.num_rows());

        // evaluate all divisors in the evaluation domain
        let divisors_evaluations =
            get_divisor_evaluations::<E>(&self.divisors, self.evaluations[0].len(), domain_offset);

        // iterate over all columns of the constraint evaluation table, divide each column
        // by the evaluations of its corresponding divisor, and add all resulting evaluations
        // together into a single vector
        for (i, (column, divisor)) in self
            .evaluations
            .into_iter()
            .zip(self.divisors.iter())
            .enumerate()
        {
            // in debug mode, make sure post-division degree of each column matches the expected
            // degree
            #[cfg(debug_assertions)]
            validate_column_degree(&column, divisor, domain_offset, column.len() - 1)?;

            acc_column(column, &divisors_evaluations[i], &mut combined_poly);
        }

        // at this point, combined_poly contains evaluations of the combined constraint polynomial;
        // we interpolate this polynomial to transform it into coefficient form.
        let inv_twiddles = fft::get_inv_twiddles::<E::BaseField>(combined_poly.len());
        fft::interpolate_poly_with_offset(&mut combined_poly, &inv_twiddles, domain_offset);

        Ok(CompositionPoly::new(combined_poly, self.trace_length))
    }

    // DEBUG HELPERS
    // --------------------------------------------------------------------------------------------

    #[cfg(debug_assertions)]
    pub fn validate_transition_degrees(&mut self) {
        // evaluate all transition constraint divisors over the constraint evaluation domain.
        // This is used later to compute
        // actual degrees of transition constraint evaluations.
        let div_values = self
            .divisors()
            .iter()
            .map(|divisor| {
                evaluate_divisor::<E::BaseField>(divisor, self.num_rows(), self.domain_offset)
            })
            .collect::<Vec<_>>();

        // collect actual degrees for all transition constraints by interpolating saved
        // constraint evaluations into polynomials and checking their degree; also
        // determine max transition constraint degree
        let mut actual_degrees = Vec::with_capacity(self.expected_transition_degrees.len());
        let mut max_degree = 0;
        let inv_twiddles = fft::get_inv_twiddles::<E::BaseField>(self.num_rows());

        // first process transition constraint evaluations for the main trace segment
        for (i, evaluations) in self.main_transition_evaluations.iter().enumerate() {
            let degree = get_transition_poly_degree(
                evaluations,
                &inv_twiddles,
                &div_values[self.main_divisor_indices[i]],
            );
            actual_degrees.push(degree);
            max_degree = core::cmp::max(max_degree, degree);
        }

        // then process transition constraint evaluations for auxiliary trace segments
        for (i, evaluations) in self.aux_transition_evaluations.iter().enumerate() {
            let degree = get_transition_poly_degree(
                evaluations,
                &inv_twiddles,
                &div_values[self.aux_divisor_indices[i]],
            );
            actual_degrees.push(degree);
            max_degree = core::cmp::max(max_degree, degree);
        }

        // make sure expected and actual degrees are equal
        assert_eq!(
            self.expected_transition_degrees, actual_degrees,
            "transition constraint degrees didn't match\nexpected: {:>3?}\nactual:   {:>3?}",
            self.expected_transition_degrees, actual_degrees
        );

        // make sure evaluation domain size does not exceed the size required by max degree
        let expected_domain_size =
            core::cmp::max(max_degree, self.trace_length + 1).next_power_of_two();
        assert_eq!(
            expected_domain_size,
            self.num_rows(),
            "incorrect constraint evaluation domain size; expected {}, but was {}",
            expected_domain_size,
            self.num_rows()
        );
    }
}

// TABLE FRAGMENTS
// ================================================================================================

pub struct EvaluationTableFragment<'a, E: FieldElement> {
    offset: usize,
    evaluations: Vec<&'a mut [E]>,

    #[cfg(debug_assertions)]
    tm_evaluations: Vec<&'a mut [E::BaseField]>,
    #[cfg(debug_assertions)]
    ta_evaluations: Vec<&'a mut [E]>,
}

impl<'a, E: FieldElement> EvaluationTableFragment<'a, E> {
    /// Returns the row at which the fragment starts.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the number of evaluation rows in the fragment.
    pub fn num_rows(&self) -> usize {
        self.evaluations[0].len()
    }

    /// Returns the number of columns in every evaluation row.
    pub fn num_columns(&self) -> usize {
        self.evaluations.len()
    }

    /// Updates a single row in the fragment with provided data.
    pub fn update_row(&mut self, row_idx: usize, row_data: &[E]) {
        for (column, &value) in self.evaluations.iter_mut().zip(row_data) {
            column[row_idx] = value;
        }
    }

    /// Updates transition evaluations row with the provided data; available only in debug mode.
    #[cfg(debug_assertions)]
    pub fn update_transition_evaluations(
        &mut self,
        row_idx: usize,
        main_evaluations: &[E::BaseField],
        aux_evaluations: &[E],
    ) {
        for (column, &value) in self.tm_evaluations.iter_mut().zip(main_evaluations) {
            column[row_idx] = value;
        }
        for (column, &value) in self.ta_evaluations.iter_mut().zip(aux_evaluations) {
            column[row_idx] = value;
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Allocates memory for a two-dimensional data structure without initializing it.
fn uninit_matrix<E: FieldElement>(num_cols: usize, num_rows: usize) -> Vec<Vec<E>> {
    unsafe { (0..num_cols).map(|_| uninit_vector(num_rows)).collect() }
}

/// Breaks the source data into a mutable set of fragments such that each fragment has the same
/// number of columns as the source data, and the number of rows equal to `num_fragments`
/// parameter.
///
/// If the source data is empty, the returned vector will contain number of empty vectors equal
/// to `num_fragments` parameter.
fn make_fragments<E: FieldElement>(
    source: &mut [Vec<E>],
    num_fragments: usize,
) -> Vec<Vec<&mut [E]>> {
    let mut result = (0..num_fragments).map(|_| Vec::new()).collect::<Vec<_>>();
    if !source.is_empty() {
        let fragment_size = source[0].len() / num_fragments;
        source.iter_mut().for_each(|column| {
            for (i, fragment) in column.chunks_mut(fragment_size).enumerate() {
                result[i].push(fragment);
            }
        });
    }
    result
}

/// Accumulates the constraint evaluation divided by the corresponding divisor to the result.
/// The divisor is already computed and inverted, therefore we simply compute
/// column[i]*divisor_evaluations[i]
fn acc_column<E: FieldElement>(column: Vec<E>, divisor_evaluations: &[E], result: &mut [E]) {
    iter_mut!(result, 1024)
        .zip(column)
        .enumerate()
        .for_each(|(i, (acc_value, value))| {
            *acc_value += value * divisor_evaluations[i];
        });
}

/// Takes a list of divisors and evaluates them over the domain
fn get_divisor_evaluations<E: FieldElement>(
    divisors: &[ConstraintDivisor<E::BaseField>],
    domain_size: usize,
    domain_offset: E::BaseField,
) -> Vec<Vec<E>> {
    // A map to save the evaluations of divisor denominator (exemption) products
    let mut evaluations_map = BTreeMap::new();
    // A map to save the inverse evaluations of divisor numerator products
    let mut inverse_evaluations_map = BTreeMap::new();

    let g_domain = E::BaseField::get_root_of_unity(domain_size.trailing_zeros());
    // iterate over divisors to get all product terms
    for divisor in divisors {
        // evaluate numerator and denominator values
        for product in divisor
            .numerator()
            .iter()
            .chain(divisor.denominator().iter())
        {
            // if the product (X^k - h) has been previously evaluated simply ignore
            if !evaluations_map.contains_key(&(product.degree(), product.coset_dlog())) {
                // otherwise check if some other term X^k has been evaluate and shift it by h
                // if not evaluate the product and save both X^k and (X^k-h) evaluations.
                // We do this since shifting is cheaper than evaluating the exponentiations
                let key = (product.degree(), 0);
                let shifted_key = (product.degree(), product.coset_dlog());
                let evaluations = evaluations_map.entry(key).or_insert_with(|| {
                    let a = product.degree() as u64;
                    let n = domain_size / a as usize;
                    let g = g_domain.exp(a.into());

                    // Compute unshifted values X^k-1 over domain
                    let mut evaluations = unsafe { uninit_vector(n) };
                    batch_iter_mut!(
                        &mut evaluations,
                        128, // min batch size
                        |batch: &mut [E], batch_offset: usize| {
                            let mut x = domain_offset
                                .exp(a.into())
                                .mul_base(g.exp((batch_offset as u64).into()));
                            for evaluation in batch.iter_mut() {
                                let val: E = x.into();
                                *evaluation = val - E::ONE;
                                x *= g;
                            }
                        }
                    );
                    evaluations
                });
                // Compute and insert shifted values
                if product.coset_dlog() != 0 {
                    let mut shifted_evaluations =
                        unsafe { uninit_vector(domain_size / product.degree()) };
                    let shift = E::ONE - product.coset_elem().into();
                    iter_mut!(shifted_evaluations, 1024).enumerate().for_each(
                        |(i, shifted_evaluation)| {
                            *shifted_evaluation = evaluations[i] + shift;
                        },
                    );
                    evaluations_map.insert(shifted_key, shifted_evaluations);
                }
            }
        }
        // TODO [divisors]: should batch these together as well

        for product in divisor.numerator() {
            let key = (product.degree(), product.coset_dlog());
            // invert and insert the values if not there already
            let _ = inverse_evaluations_map.entry(key).or_insert_with(|| {
                batch_inversion(
                    evaluations_map
                        .get(&(product.degree(), product.coset_dlog()))
                        .unwrap(), // should never fail
                )
            });
        }
    }

    // TODO [divisors]: rewrite parallelizable
    // compute divisor evaluations using the saved values of the dictionaries
    let mut divisors_evaluations = vec![];
    for divisor in divisors.iter() {
        // result is the final divisor evaluation

        let mut result = vec![E::ONE; domain_size];
        for product in divisor.denominator() {
            let key = (product.degree(), product.coset_dlog());
            // the values considered for the product
            let z = evaluations_map.get(&key).unwrap();
            for i in 0..domain_size {
                result[i] *= z[i % z.len()];
            }
        }
        for product in divisor.numerator() {
            let key = (product.degree(), product.coset_dlog());
            // the values considered for the product inverted
            let z = inverse_evaluations_map.get(&key).unwrap();
            for i in 0..domain_size {
                result[i] *= z[i % z.len()]
            }
        }
        divisors_evaluations.push(result);
    }

    divisors_evaluations
}

// DEBUG HELPERS
// ================================================================================================

/// Returns evaluation degrees of all transition constraints.
///
/// An evaluation degree is defined as degree of transition constraints in the context of a given
/// execution trace accounting for constraint divisor degree. For most constraints, this degree is
/// computed as `([trace_length - 1] * [constraint degree]) - [divisor degree]`. However, for
/// constraints which rely on periodic columns this computation is slightly more complex.
///
/// The general idea is that evaluation degree is the degree of rational function `C(x) / z(x)`,
/// where `C(x)` is the constraint polynomial and `z(x)` is the divisor polynomial.
#[cfg(debug_assertions)]
fn build_transition_constraint_degrees<E: FieldElement>(
    constraints: &TransitionConstraints<E>,
    trace_length: usize,
) -> Vec<usize> {
    let mut result = Vec::new();

    for (idx, degree) in constraints.main_constraint_degrees().iter().enumerate() {
        let divisor_idx = constraints.main_constraints_divisors()[idx];
        result.push(
            degree.get_evaluation_degree(trace_length)
                - constraints.divisors()[divisor_idx].degree(),
        )
    }

    for (idx, degree) in constraints.aux_constraint_degrees().iter().enumerate() {
        let divisor_idx = constraints.aux_constraints_divisors()[idx];
        result.push(
            degree.get_evaluation_degree(trace_length)
                - constraints.divisors()[divisor_idx].degree(),
        )
    }

    result
}

/// Computes the actual degree of a transition polynomial described by the provided evaluations.
///
/// The degree is computed as follows:
/// - First, we divide the polynomial evaluations by the evaluations of transition constraint
///   divisor (`div_values`). This is needed because it is possible for the numerator portions of
///   transition constraints to have a degree which is larger than the size of the evaluation
///   domain (and thus, interpolating the numerator would yield an incorrect result). However,
///   once the divisor values are divided out, the degree of the resulting polynomial should be
///   smaller than the size of the evaluation domain, and thus, we can interpolate safely.
/// - Then, we interpolate the polynomial over the domain specified by `inv_twiddles`.
/// - And finally, we get the degree from the interpolated polynomial.
#[cfg(debug_assertions)]
fn get_transition_poly_degree<E: FieldElement>(
    evaluations: &[E],
    inv_twiddles: &[E::BaseField],
    div_values: &[E::BaseField],
) -> usize {
    let mut evaluations = evaluations
        .iter()
        .zip(div_values)
        .map(|(&c, &d)| c / E::from(d))
        .collect::<Vec<_>>();
    fft::interpolate_poly(&mut evaluations, inv_twiddles);
    math::polynom::degree_of(&evaluations)
}

/// Makes sure that the post-division degree of the polynomial matches the expected degree
#[cfg(debug_assertions)]
fn validate_column_degree<B: StarkField, E: FieldElement<BaseField = B>>(
    column: &[E],
    divisor: &ConstraintDivisor<B>,
    domain_offset: B,
    expected_degree: usize,
) -> Result<(), ProverError> {
    // build domain for divisor evaluation, and evaluate it over this domain
    let div_values = evaluate_divisor(divisor, column.len(), domain_offset);

    // divide column values by the divisor
    let mut evaluations = column
        .iter()
        .zip(div_values)
        .map(|(&c, d)| c / d)
        .collect::<Vec<_>>();

    // interpolate evaluations into a polynomial in coefficient form
    let inv_twiddles = fft::get_inv_twiddles::<B>(evaluations.len());
    fft::interpolate_poly_with_offset(&mut evaluations, &inv_twiddles, domain_offset);
    let poly = evaluations;

    if expected_degree != math::polynom::degree_of(&poly) {
        return Err(ProverError::MismatchedConstraintPolynomialDegree(
            expected_degree,
            math::polynom::degree_of(&poly),
        ));
    }
    Ok(())
}

/// Evaluates constraint divisor over the specified domain. This is similar to [get_inv_evaluation]
/// function above but uses a more straight-forward but less efficient evaluation methodology and
/// also does not invert the results.
#[cfg(debug_assertions)]
fn evaluate_divisor<E: FieldElement>(
    divisor: &ConstraintDivisor<E::BaseField>,
    domain_size: usize,
    domain_offset: E::BaseField,
) -> Vec<E> {
    let g = E::BaseField::get_root_of_unity(domain_size.trailing_zeros());
    let domain = math::get_power_series_with_offset(g, domain_offset, domain_size);
    domain
        .into_iter()
        .map(|x| E::from(divisor.evaluate_at(x)))
        .collect()
}
