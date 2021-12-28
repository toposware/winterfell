// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::StarkDomain;

mod trace_table;
use air::{Air, EvaluationFrame, TraceInfo};
use math::{fft, polynom, StarkField};
pub use trace_table::TraceTable;

mod poly_table;
pub use poly_table::TracePolyTable;

mod execution_trace;
pub use execution_trace::{ExecutionTrace, ExecutionTraceFragment};

use utils::{collections::Vec, iter_mut};

#[cfg(feature = "concurrent")]
use utils::iterators::*;

#[cfg(test)]
mod tests;

// TRACE TRAIT
// ================================================================================================

// TODO: add docs
pub trait Trace<B: StarkField>: Sized {
    // REQUIRED METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns number of columns in the trace.
    fn width(&self) -> usize;

    /// Returns the number of rows in this trace.
    fn length(&self) -> usize;

    /// Returns metadata associated with this trace.
    fn meta(&self) -> &[u8];

    /// Returns value of the cell in the specified column at the specified row.
    fn get(&self, col_idx: usize, row_idx: usize) -> B;

    /// Reads a single row of this trace at the specified index into the specified target.
    fn read_row_into(&self, step: usize, target: &mut [B]);

    /// Transforms this trace into a vector of columns containing trace data.
    fn into_columns(self) -> Vec<Vec<B>>;

    // PROVIDED METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns trace info for this trace.
    fn get_info(&self) -> TraceInfo {
        TraceInfo::with_meta(self.width(), self.length(), self.meta().to_vec())
    }

    // VALIDATION
    // --------------------------------------------------------------------------------------------
    /// Checks if this trace is valid against the specified AIR, and panics if not.
    ///
    /// NOTE: this is a very expensive operation and is intended for use only in debug mode.
    fn validate<A: Air<BaseElement = B>>(&self, air: &A) {
        // TODO: eventually, this should return errors instead of panicking

        // make sure the width align; if they don't something went terribly wrong
        assert_eq!(
            self.width(),
            air.trace_width(),
            "inconsistent trace width: expected {}, but was {}",
            self.width(),
            air.trace_width()
        );

        // --- 1. make sure the assertions are valid ----------------------------------------------
        for assertion in air.get_assertions() {
            assertion.apply(self.length(), |step, value| {
                assert!(
                    value == self.get(assertion.register(), step),
                    "trace does not satisfy assertion trace({}, {}) == {}",
                    assertion.register(),
                    step,
                    value
                );
            });
        }

        // --- 2. make sure this trace satisfies all transition constraints -----------------------

        // collect the info needed to build periodic values for a specific step
        let g = air.trace_domain_generator();
        let periodic_values_polys = air.get_periodic_column_polys();
        let mut periodic_values = vec![B::ZERO; periodic_values_polys.len()];

        // initialize buffers to hold evaluation frames and results of constraint evaluations
        let mut x = B::ONE;
        let mut ev_frame = EvaluationFrame::new(self.width());
        let mut evaluations = vec![B::ZERO; air.num_transition_constraints()];

        for step in 0..self.length() - 1 {
            // build periodic values
            for (p, v) in periodic_values_polys.iter().zip(periodic_values.iter_mut()) {
                let num_cycles = air.trace_length() / p.len();
                let x = x.exp((num_cycles as u32).into());
                *v = polynom::eval(p, x);
            }

            // build evaluation frame
            self.read_row_into(step, ev_frame.current_mut());
            self.read_row_into(step + 1, ev_frame.next_mut());

            // evaluate transition constraints
            air.evaluate_transition(&ev_frame, &periodic_values, &mut evaluations);

            // make sure all constraints evaluated to ZERO
            for (i, &evaluation) in evaluations.iter().enumerate() {
                assert!(
                    evaluation == B::ZERO,
                    "transition constraint {} did not evaluate to ZERO at step {}",
                    i,
                    step
                );
            }

            // update x coordinate of the domain
            x *= g;
        }
    }

    // LOW-DEGREE EXTENSION
    // --------------------------------------------------------------------------------------------
    /// Extends all columns of the trace table to the length of the LDE domain.
    ///
    /// The extension is done by first interpolating each register into a polynomial over the
    /// trace domain, and then evaluating the polynomial over the LDE domain.
    fn extend(self, domain: &StarkDomain<B>) -> (TraceTable<B>, TracePolyTable<B>) {
        assert_eq!(
            self.length(),
            domain.trace_length(),
            "inconsistent trace length"
        );
        // build and cache trace twiddles for FFT interpolation; we do it here so that we
        // don't have to rebuild these twiddles for every register.
        let inv_twiddles = fft::get_inv_twiddles::<B>(domain.trace_length());

        // extend all registers; the extension procedure first interpolates register traces into
        // polynomials (in-place), then evaluates these polynomials over a larger domain, and
        // then returns extended evaluations.
        let mut columns = self.into_columns();
        let extended_trace = iter_mut!(columns)
            .map(|register_trace| extend_column(register_trace, domain, &inv_twiddles))
            .collect();

        (
            TraceTable::new(extended_trace, domain.trace_to_lde_blowup()),
            TracePolyTable::new(columns),
        )
    }
}

// HELPER FUNCTIONS
// ================================================================================================

#[inline(always)]
fn extend_column<B: StarkField>(
    column: &mut [B],
    domain: &StarkDomain<B>,
    inv_twiddles: &[B],
) -> Vec<B> {
    let domain_offset = domain.offset();
    let twiddles = domain.trace_twiddles();
    let blowup_factor = domain.trace_to_lde_blowup();

    // interpolate register trace into a polynomial; we do this over the un-shifted trace_domain
    fft::interpolate_poly(column, inv_twiddles);

    // evaluate the polynomial over extended domain; the domain may be shifted by the
    // domain_offset
    fft::evaluate_poly_with_offset(column, twiddles, domain_offset, blowup_factor)
}
