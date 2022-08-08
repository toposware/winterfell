// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{ecc::{GENERATOR, AFFINE_POINT_WIDTH, ECPoint, apply_point_doubling}, field};
use core::cmp::Ordering;
use winterfell::math::{curves::curve_f63::Scalar, fields::f63::BaseElement, FieldElement};

// CONSTANT
// ================================================================================================

pub fn get_constant_points() -> [ECPoint; 16] {
    let mut points = [[BaseElement::ZERO; AFFINE_POINT_WIDTH]; 16];
    let mut state = GENERATOR;
    points[0] = GENERATOR;
    apply_point_doubling(&mut points[1]);
    apply_point_doubling(&mut points[2]);
    apply_point_doubling(&mut points[3]);
    apply_point_doubling(&mut points[4]);
    apply_point_doubling(&mut points[5]);
    apply_point_doubling(&mut points[6]);
    apply_point_doubling(&mut points[7]);
    apply_point_doubling(&mut points[8]);
    apply_point_doubling(&mut points[9]);
    apply_point_doubling(&mut points[10]);
    apply_point_doubling(&mut points[12]);
    apply_point_doubling(&mut points[13]);
    apply_point_doubling(&mut points[14]);
    apply_point_doubling(&mut points[15]);
    points
}