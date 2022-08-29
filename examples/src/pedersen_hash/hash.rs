// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use winterfell::math::{fields::f63::BaseElement, FieldElement};

use super::ecc::{GENERATOR, TWICE_THE_GENERATOR, ECPoint, compute_add_affine, AFFINE_POINT_WIDTH};

// CONSTANT
// ================================================================================================

pub const fn get_intial_constant_point() -> ECPoint {
    GENERATOR
}

pub fn get_constant_points<const N_POINTS: usize>() -> [ECPoint; N_POINTS] {
    let mut points = [GENERATOR; N_POINTS];
    points[0] = TWICE_THE_GENERATOR;
    for i in 1..N_POINTS {
        let mut rhs = [BaseElement::ZERO; AFFINE_POINT_WIDTH];
        rhs.copy_from_slice(&points[i-1][0..AFFINE_POINT_WIDTH]);
        compute_add_affine(&mut points[i], &rhs);
    }
    points
}
