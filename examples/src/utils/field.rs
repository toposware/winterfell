// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{are_equal, is_binary, EvaluationResult};
use winterfell::math::{fields::f63::BaseElement, FieldElement};

// TRACE
// ================================================================================================

/// Apply a step of double-and-add in the field when filling up the execution trace.
pub(crate) fn apply_double_and_add_step(
    state: &mut [BaseElement],
    value_position: usize,
    bit_position: usize,
) {
    state[value_position] = state[value_position].double() + state[bit_position];
}

// CONSTRAINTS
// ================================================================================================

/// Enforce a step of double-and-add in the field, given two registers:
///  - an accumulated value, starting from zero
///  - a binary value, indicating whether we add after doubling or not
/// and enforce binary constraint on the bit decomposition register
pub(crate) fn enforce_double_and_add_step<E: FieldElement>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    value_position: usize,
    bit_position: usize,
    flag: E,
) {
    let mut step1 = current[value_position];
    let step2 = next[value_position];

    // We can directly add next[bit_position], as its value is either E::ZERO or E::ONE
    step1 = step1.double() + next[bit_position];

    // make sure that the results are equal
    result.agg_constraint(value_position, flag, are_equal(step2, step1));

    // enforce that the binary input is indeed, binary
    result.agg_constraint(bit_position, flag, is_binary(next[bit_position]));
}

/// Enforce a step of double-and-add in the field, with an
/// already constrained binary input.
pub(crate) fn enforce_double_and_add_step_constrained<E: FieldElement>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    value_position: usize,
    bit_position: usize,
    flag: E,
) {
    let mut step1 = current[value_position];
    let step2 = next[value_position];

    // We can directly add next[bit_position], as its value is either E::ZERO or E::ONE
    step1 = step1.double() + next[bit_position];

    // make sure that the results are equal
    result.agg_constraint(value_position, flag, are_equal(step2, step1));
}