// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use air::Air;
use math::{batch_inversion, fft, get_power_series_with_offset, StarkField};
use utils::{
    collections::{BTreeMap, BTreeSet, Vec},
    iter_mut, uninit_vector,
};

#[cfg(feature = "concurrent")]
use utils::iterators::*;

// CONSANTS
// ================================================================================================

// Defines the cost of an inversion in terms of multiplications.
const INVERSION_COST: usize = 50;

// ================================================================================================

pub struct PeriodicValueTable<B: StarkField> {
    values: Vec<B>,
    length: usize,
    width: usize,
}

impl<B: StarkField> PeriodicValueTable<B> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Builds a table of periodic column values for the specified AIR. The table contains expanded
    /// values of all periodic columns normalized to the same length. This enables simple lookup
    /// into the able using step index of the constraint evaluation domain.
    pub fn new<A: Air<BaseField = B>>(air: &A) -> PeriodicValueTable<B> {
        // get a list of polynomials describing periodic columns from AIR. if there are no
        // periodic columns return an empty table
        let polys = air.get_periodic_column_polys();
        if polys.is_empty() {
            return PeriodicValueTable {
                values: Vec::new(),
                length: 0,
                width: 0,
            };
        }

        // determine the size of the biggest polynomial in the set. unwrap is OK here
        // because if we get here, there must be at least one polynomial in the set.
        let max_poly_size = polys.iter().max_by_key(|p| p.len()).unwrap().len();

        // cache twiddles used for polynomial evaluation here so that we don't have to re-build
        // them for polynomials of the same size
        let mut twiddle_map = BTreeMap::new();

        let evaluations = polys
            .iter()
            .map(|poly| {
                let poly_size = poly.len();
                let num_cycles = (air.trace_length() / poly_size) as u64;
                let offset = air.domain_offset().exp(num_cycles.into());
                let twiddles = twiddle_map
                    .entry(poly_size)
                    .or_insert_with(|| fft::get_twiddles(poly_size));

                fft::evaluate_poly_with_offset(poly, twiddles, offset, air.ce_blowup_factor())
            })
            .collect::<Vec<_>>();

        // allocate memory to hold all expanded values and copy polynomial evaluations into the
        // table in such a way that values for the same row are adjacent to each other.
        let row_width = polys.len();
        let column_length = max_poly_size * air.ce_blowup_factor();
        let mut values = unsafe { uninit_vector(row_width * column_length) };
        for i in 0..column_length {
            for (j, column) in evaluations.iter().enumerate() {
                values[i * row_width + j] = column[i % column.len()];
            }
        }

        PeriodicValueTable {
            values,
            length: column_length,
            width: row_width,
        }
    }
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Builds a table of custom divisor column values for the specified AIR. The table contains expanded
    /// values of all custom divisors columns normalized to the same length. This enables simple lookup
    /// into the able using step index of the constraint evaluation domain.
    pub fn from_custom_divisors<A: Air<BaseField = B>>(air: &A) -> PeriodicValueTable<B> {
        // get a list of polynomials describing periodic columns from AIR. if there are no
        // periodic columns return an empty table
        let polys = air.get_custom_divisors();
        if polys.is_empty() {
            return PeriodicValueTable {
                values: Vec::new(),
                length: 0,
                width: 0,
            };
        }

        // determine the size of the biggest polynomial in the set. unwrap is OK here
        // because if we get here, there must be at least one polynomial in the set.
        let max_poly_size = polys.iter().max_by_key(|p| p.0).unwrap().0;

        // size of the evaluation table
        let row_width = polys.len();
        let column_length = max_poly_size * air.ce_blowup_factor();

        // constraint evaluation domain
        let domain_size = air.ce_domain_size();

        // initialize a vector to store the values
        let mut values = unsafe { uninit_vector(row_width * column_length) };

        let mut exponentiations_map = BTreeMap::new();

        // evaluate numerator X^n - 1
        let numerator_evaluations = get_power_series_with_offset(
            B::get_root_of_unity(domain_size.trailing_zeros())
                .exp((air.trace_length() as u64).into()),
            air.domain_offset().exp((air.trace_length() as u64).into()),
            domain_size / air.trace_length(),
        )
        .iter()
        .map(|x| *x - B::ONE)
        .collect::<Vec<_>>();

        let mut divisor_offsets = Vec::with_capacity(polys.len());
        // compute offset points for each divisor
        for (period, offsets) in polys.iter() {
            // We either compute the given or the complementary offsets depending on how
            // we evaluate the divisor
            let num_cycles = air.trace_length() / period;
            if offsets.len() + INVERSION_COST < period - offsets.len() {
                // we use inversions in this case
                let g_offsets = offsets
                    .iter()
                    .map(|offset| {
                        air.trace_domain_generator()
                            .exp(((num_cycles * offset) as u64).into())
                    })
                    .collect::<Vec<_>>();
                divisor_offsets.push(g_offsets);
            } else {
                // we use multiplications in this case
                let mut c_offsets = BTreeSet::new();
                for i in 0..*period {
                    c_offsets.insert(i);
                }
                for offset in offsets {
                    c_offsets.remove(offset);
                }
                let g_offsets = c_offsets
                    .iter()
                    .map(|offset| {
                        air.trace_domain_generator()
                            .exp(((num_cycles * offset) as u64).into())
                    })
                    .collect::<Vec<_>>();
                divisor_offsets.push(g_offsets);
            }
        }

        // evaluate each divisor and save the result.
        // evaluation is done either with multiplications or with inversions
        for (j, (period, offsets)) in polys.iter().enumerate() {
            // We can evaluate the divisor in two ways:
            //      1. compute X^n-1 / \Prod X^k - offset.
            //         This involves offsets.len() multiplication and one iversion per point
            //      2. compute \Prod X^k - c_offset where c_offset are elements not
            //         included in the offset
            //         This involves period - offsets.len() multiplication
            let divisor_evaluations = if offsets.len() + INVERSION_COST < period - offsets.len() {
                Self::evaluate_custom_divisor_with_inversions(
                    air,
                    &numerator_evaluations,
                    &mut exponentiations_map,
                    period,
                    &divisor_offsets[j],
                )
            } else {
                Self::evaluate_custom_divisor_with_multiplications(
                    air,
                    &mut exponentiations_map,
                    period,
                    &divisor_offsets[j],
                )
            };

            // record the results in the table
            for i in 0..column_length {
                values[i * row_width + j] = divisor_evaluations[i % divisor_evaluations.len()];
            }
        }

        PeriodicValueTable {
            values,
            length: column_length,
            width: row_width,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    pub fn is_empty(&self) -> bool {
        self.width == 0
    }

    pub fn get_row(&self, ce_step: usize) -> &[B] {
        if self.is_empty() {
            &[]
        } else {
            let start = (ce_step % self.length) * self.width;
            &self.values[start..start + self.width]
        }
    }

    // HELPER FUNCTIONS

    /// Evaluates a custom divisor described by a period $p$ and a set of offsets. The constraint
    /// is forced to hold on steps $k\cdot p + i$ for each offset $i$.    
    ///
    /// The divisor is evaluated by evaluating $\frac{X^n-1}{\Prod X^{n/p}-g^i}$ where i are the
    /// given offsets.
    pub fn evaluate_custom_divisor_with_inversions<A: Air<BaseField = B>>(
        air: &A,
        numerator_evaluations: &[B],
        exponentiations_map: &mut BTreeMap<usize, Vec<B>>,
        period: &usize,
        offsets: &[B],
    ) -> Vec<B> {
        // constraint evaluation domain
        let domain_size = air.ce_domain_size();

        let num_cycles = air.trace_length() / period;

        // number of elements needed elements to evaluate denominator
        let denominator_evals_size = domain_size / num_cycles;

        // evaluate x^{trace_length/period} if not already evaluated
        let exponentiations = exponentiations_map.entry(*period).or_insert_with(|| {
            get_power_series_with_offset(
                B::get_root_of_unity(domain_size.trailing_zeros()).exp((num_cycles as u64).into()),
                air.domain_offset().exp((num_cycles as u64).into()),
                denominator_evals_size,
            )
        });

        //evaluate the denominator

        // initialize values to ONE
        let mut denominator_evaluations = vec![B::ONE; denominator_evals_size];

        iter_mut!(denominator_evaluations, 128)
            .enumerate()
            .for_each(|(i, value)| {
                for g_offset in offsets {
                    *value *= exponentiations[i] - *g_offset;
                }
            });

        // invert the numerator evaluations
        let mut evaluations = batch_inversion(&denominator_evaluations);

        // multiply the inverse values with the numerator to get the final divisor evaluation
        iter_mut!(evaluations, 128)
            .enumerate()
            .for_each(|(i, evaluation)| {
                *evaluation *= numerator_evaluations[i % numerator_evaluations.len()];
            });
        evaluations
    }

    /// Evaluates a custom divisor described by a period $p$ and a set of offsets. The constraint
    /// is forced to hold on steps $k\cdot p + i$ for each offset $i$.
    ///
    /// The divisor is evaluated by evaluating $\Prod X^{n/p}-g^i$ where i are the
    /// elements not included in the given offsets.
    pub fn evaluate_custom_divisor_with_multiplications<A: Air<BaseField = B>>(
        air: &A,
        exponentiations_map: &mut BTreeMap<usize, Vec<B>>,
        period: &usize,
        offsets: &[B],
    ) -> Vec<B> {
        // constraint evaluation domain
        let domain_size = air.ce_domain_size();

        let num_cycles = air.trace_length() / period;

        // number of elements needed elements to evaluate denominator
        let denominator_evals_size = domain_size / num_cycles;

        // evaluate x^{trace_length/period} if not already evaluated
        let exponentiations = exponentiations_map.entry(*period).or_insert_with(|| {
            get_power_series_with_offset(
                B::get_root_of_unity(domain_size.trailing_zeros()).exp((num_cycles as u64).into()),
                air.domain_offset().exp((num_cycles as u64).into()),
                domain_size / num_cycles,
            )
        });

        //evaluate the denominator

        let mut denominator_evaluations = vec![B::ONE; denominator_evals_size];

        iter_mut!(denominator_evaluations, 128)
            .enumerate()
            .for_each(|(i, value)| {
                for g_offset in offsets {
                    *value *= exponentiations[i] - *g_offset;
                }
            });

        denominator_evaluations
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use crate::tests::MockAir;
    use air::Air;
    use math::{
        fields::f128::BaseElement, get_power_series_with_offset, log2, polynom, FieldElement,
        StarkField,
    };
    use utils::collections::Vec;

    #[test]
    fn periodic_value_table() {
        let trace_length = 32;

        // instantiate AIR with 2 periodic columns
        let col1 = vec![1u128, 2]
            .into_iter()
            .map(BaseElement::new)
            .collect::<Vec<_>>();
        let col2 = vec![3u128, 4, 5, 6]
            .into_iter()
            .map(BaseElement::new)
            .collect::<Vec<_>>();
        let air = MockAir::with_periodic_columns(vec![col1, col2], trace_length);

        // build a table of periodic values
        let table = super::PeriodicValueTable::new(&air);

        assert_eq!(2, table.width);
        assert_eq!(4 * air.ce_blowup_factor(), table.length);

        let polys = air.get_periodic_column_polys();
        let domain = build_ce_domain(air.ce_domain_size(), air.domain_offset());

        // build expected values by evaluating polynomials over shifted ce_domain
        let expected = polys
            .iter()
            .map(|poly| {
                let num_cycles = trace_length / poly.len();
                domain
                    .iter()
                    .map(|&x| {
                        let x = x.exp((num_cycles as u32).into());
                        polynom::eval(poly, x)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // build actual values by recording rows of the table at each step of ce_domain
        let mut actual = vec![Vec::new(), Vec::new()];
        for i in 0..air.ce_domain_size() {
            let row = table.get_row(i);
            actual[0].push(row[0]);
            actual[1].push(row[1]);
        }

        assert_eq!(expected, actual);
    }

    fn build_ce_domain(domain_size: usize, domain_offset: BaseElement) -> Vec<BaseElement> {
        let g = BaseElement::get_root_of_unity(log2(domain_size));
        get_power_series_with_offset(g, domain_offset, domain_size)
    }
}
