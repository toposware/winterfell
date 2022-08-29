// Copyright (c) 2021-2022 Toposware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{are_equal, is_binary, not, EvaluationResult};
use winterfell::math::{fields::f63::BaseElement, FieldElement};

use std::iter::repeat;

pub type ECPoint = [BaseElement; AFFINE_POINT_WIDTH];

// CONSTANTS
// ================================================================================================

/// The length of a point coordinate
pub const POINT_COORDINATE_WIDTH: usize = 6;
/// The length of an AffinePoint
pub const AFFINE_POINT_WIDTH: usize = POINT_COORDINATE_WIDTH * 2;
/// The length of a ProjectivePoint
pub const PROJECTIVE_POINT_WIDTH: usize = POINT_COORDINATE_WIDTH * 3;

/// Specifies the affine coordinates of the curve generator G
pub const GENERATOR: ECPoint = [
    BaseElement::from_raw_unchecked(0xf6798582c92ece1),
    BaseElement::from_raw_unchecked(0x2b7c30a4c7d886c0),
    BaseElement::from_raw_unchecked(0x1269cdae98dc2fd0),
    BaseElement::from_raw_unchecked(0x11b78ef6c71c6132),
    BaseElement::from_raw_unchecked(0x3ac2244dfc47537),
    BaseElement::from_raw_unchecked(0x36dfeea4b9051daf),
    BaseElement::from_raw_unchecked(0x334807e450d55e2f),
    BaseElement::from_raw_unchecked(0x200a54d42b84bd17),
    BaseElement::from_raw_unchecked(0x271af7bb20ab32e1),
    BaseElement::from_raw_unchecked(0x3df7b90927efc7ec),
    BaseElement::from_raw_unchecked(0xab8bbf4a53af6a0),
    BaseElement::from_raw_unchecked(0xe13dca26b2ac6ab),
];

pub const TWICE_THE_GENERATOR: ECPoint = [
    BaseElement::from_raw_unchecked(0x1ba2d52806f212a),
    BaseElement::from_raw_unchecked(0x5e9353a4e8225c8),
    BaseElement::from_raw_unchecked(0x13e92423fef3bc2d),
    BaseElement::from_raw_unchecked(0x241081e7ae1db310),
    BaseElement::from_raw_unchecked(0x29f0073c3351026b),
    BaseElement::from_raw_unchecked(0x11233fe9eb7285c0),
    BaseElement::from_raw_unchecked(0x3a19dfba18e15ed5),
    BaseElement::from_raw_unchecked(0x3691eb6949fca20b),
    BaseElement::from_raw_unchecked(0x3ea42cb9ad7430ab),
    BaseElement::from_raw_unchecked(0x1b840f91119a2eb3),
    BaseElement::from_raw_unchecked(0x1b94f8ccdafc47ba),
    BaseElement::from_raw_unchecked(0x19e92e12c3a9cfa),
];

pub const B3: [BaseElement; POINT_COORDINATE_WIDTH] = [
    BaseElement::new(4580716109223965136),
    BaseElement::new(2805468717395796313),
    BaseElement::new(1114868343634801550),
    BaseElement::new(2558072281956999041),
    BaseElement::new(1087679150666117746),
    BaseElement::new(3602598603028951788),
];

// TRACE
// ================================================================================================

/// Apply a point doubling.
pub(crate) fn apply_point_doubling(state: &mut [BaseElement]) {
    compute_double(state);
}

/// Apply a point addition between the current `state` registers with a given point.
pub(crate) fn apply_point_addition(state: &mut [BaseElement], point: &[BaseElement]) {
    if state[PROJECTIVE_POINT_WIDTH] == BaseElement::ONE {
        compute_add(state, point)
    };
    if state[PROJECTIVE_POINT_WIDTH] == BaseElement::ONE {
        compute_add_mixed(state, point)
    };
}

// CONSTRAINTS
// ================================================================================================

/// When flag = 1, checks if the constraints for performing a point addition in affine coordinates
/// are satisfied `point, lhs, rhs` and `slope`. + represents adition over the curve group, .
/// This function returns a vector of base field elements being all zero if and only if point `point` is indeed
/// the result of adding `lhs` with `rhs`.
pub(crate) fn enforce_point_addition_affine<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    lhs: &[E],
    rhs: &[E],
    slope: &[E],
    point: &[E],
    flag: E,
) {
    let mut target = [E::ZERO; AFFINE_POINT_WIDTH];

    let x1 = &lhs[0..POINT_COORDINATE_WIDTH];
    let x2 = &rhs[0..POINT_COORDINATE_WIDTH];

    let y1 = &lhs[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];
    let y2 = &rhs[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];

    let mut slope_witness = sub_fp6(&x2, &x1);
    slope_witness = mul_fp6(slope, &slope_witness);
    slope_witness = sub_fp6(
        &slope_witness, 
        &sub_fp6(y2, y1));
    let slope_witness = slope_witness.iter().zip(repeat(flag))
        .map(
            |(&slope_witness, flag)| flag*slope_witness
        ).collect::<Vec<_>>();
    result[0..POINT_COORDINATE_WIDTH].copy_from_slice(&slope_witness);

    target.copy_from_slice(lhs);
    compute_add_affine_with_slope(&mut target, rhs, slope);

    // Make sure that the results are equal
    for i in 0..AFFINE_POINT_WIDTH {
        result.agg_constraint(
            i + POINT_COORDINATE_WIDTH, 
            flag, 
            are_equal(target[i], point[i]));
    }
}

#[test]
fn test_point_addition_affine() {
    let twice_the_generator = [
        BaseElement::new(0x7f4c1bfc52278ad8),
        BaseElement::new(0xfa8e921f7580e371),
        BaseElement::new(0x97252bf35d1c7668),
        BaseElement::new(0xe6d0901604cae95a),
        BaseElement::new(0xae36bba2ad2ee0d7),
        BaseElement::new(0x194b4e35a2a9c77),
        BaseElement::new(0x144045efbce03ef8),
        BaseElement::new(0x8e5fe3f66f8b370d),
        BaseElement::new(0x3d54df63b96bfd20),
        BaseElement::new(0x2418219e37948caa),
        BaseElement::new(0xd4c1a40432582552),
        BaseElement::new(0x367b029f5f146e3d)
    ];
    let mut slope = [BaseElement::ZERO; POINT_COORDINATE_WIDTH];
    compute_slope(&mut slope, &GENERATOR, &GENERATOR);
    let mut result = [BaseElement::ZERO; AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1];
    enforce_point_addition_affine(
        &mut result,
        &GENERATOR,
        &GENERATOR,
        &slope,
        &twice_the_generator,
        BaseElement::ONE
    );
    assert_eq!(result, [BaseElement::ZERO; AFFINE_POINT_WIDTH + POINT_COORDINATE_WIDTH + 1])
}


/// When flag = 1, enforces constraints for performing a point doubling.
pub(crate) fn enforce_point_doubling<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    flag: E,
) {
    let mut step1 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step1.copy_from_slice(&current[0..PROJECTIVE_POINT_WIDTH]);

    let mut step2 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step2.copy_from_slice(&next[0..PROJECTIVE_POINT_WIDTH]);

    compute_double(&mut step1);

    // Make sure that the results are equal
    for i in 0..PROJECTIVE_POINT_WIDTH {
        result.agg_constraint(i, flag, are_equal(step2[i], step1[i]));
    }

    // Enforce that the last register for conditional addition is indeed binary
    result.agg_constraint(
        PROJECTIVE_POINT_WIDTH,
        flag,
        is_binary(current[PROJECTIVE_POINT_WIDTH]),
    );
}

/// When flag = 1, enforces constraints for performing a mixed point addition
/// between current and point.
pub(crate) fn enforce_point_addition_mixed<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    point: &[E],
    flag: E,
) {
    let mut step1 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step1.copy_from_slice(&current[0..PROJECTIVE_POINT_WIDTH]);

    let mut step2 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step2.copy_from_slice(&next[0..PROJECTIVE_POINT_WIDTH]);

    compute_add_mixed(&mut step1, point);
    let adding_bit = current[PROJECTIVE_POINT_WIDTH];

    for i in 0..PROJECTIVE_POINT_WIDTH {
        result.agg_constraint(
            i,
            flag,
            are_equal(
                step2[i],
                adding_bit * step1[i] + not(adding_bit) * current[i],
            ),
        );
    }

    // Ensure proper duplication of the binary decomposition
    result.agg_constraint(
        PROJECTIVE_POINT_WIDTH,
        flag,
        are_equal(
            current[PROJECTIVE_POINT_WIDTH],
            next[PROJECTIVE_POINT_WIDTH],
        ),
    );
}

/// When flag = 1, enforces constraints for performing a point addition
/// between current and point in projective coordinates.
///
/// In the current implementation, this is being used only once, at the final step,
/// so we add a division of register 0 by register 2 to obtain the final affine
/// x coordinate (computations are being done internally in projective coordinates)
pub(crate) fn enforce_point_addition_reduce_x<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    point: &[E],
    flag: E,
) {
    let mut step1 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step1.copy_from_slice(&current[0..PROJECTIVE_POINT_WIDTH]);

    let mut step2 = [E::ZERO; PROJECTIVE_POINT_WIDTH];
    step2.copy_from_slice(&next[0..PROJECTIVE_POINT_WIDTH]);

    compute_add(&mut step1, point);

    let x_z = mul_fp6(
        &step2[0..POINT_COORDINATE_WIDTH],
        &step1[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH],
    );

    for i in 0..POINT_COORDINATE_WIDTH {
        result.agg_constraint(i, flag, are_equal(x_z[i], step1[i]));
    }
    for i in POINT_COORDINATE_WIDTH..PROJECTIVE_POINT_WIDTH {
        result.agg_constraint(i, flag, are_equal(step2[i], step1[i]));
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Compute the double of the current point, stored as [X,Y,Z].
/// Doubling is computed as:
///
/// `X2 = 2XY(Y^2 - 2XZ - 3BZ^2) - 2YZ(X^2 + 6BXZ - Z^2)`
///
/// `Y2 = (Y^2 + 2XZ + 3BZ^2) (Y^2 - 2XZ - 3BZ^2) + (3X^2 + Z^2) (X^2 + 6BXZ - Z^2)`
///
/// `Z2 = 8Y^3.Z`
#[inline(always)]
fn compute_double<E: FieldElement + From<BaseElement>>(state: &mut [E]) {
    let self_x = &state[0..POINT_COORDINATE_WIDTH];
    let self_y = &state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];
    let self_z = &state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH];

    let b3 = [
        E::from(B3[0]),
        E::from(B3[1]),
        E::from(B3[2]),
        E::from(B3[3]),
        E::from(B3[4]),
        E::from(B3[5]),
    ];

    let t0 = square_fp6(self_x);
    let t1 = square_fp6(self_y);
    let t2 = square_fp6(self_z);

    let t3 = mul_fp6(self_x, self_y);
    let t3 = double_fp6(&t3);
    let z3 = mul_fp6(self_x, self_z);

    let z3 = double_fp6(&z3);
    let y3 = mul_fp6(&b3, &t2);

    let y3 = add_fp6(&z3, &y3);
    let x3 = sub_fp6(&t1, &y3);
    let y3 = add_fp6(&t1, &y3);

    let y3 = mul_fp6(&x3, &y3);
    let x3 = mul_fp6(&t3, &x3);
    let z3 = mul_fp6(&b3, &z3);

    let t3 = sub_fp6(&t0, &t2);

    let t3 = add_fp6(&t3, &z3);
    let z3 = double_fp6(&t0);
    let t0 = add_fp6(&z3, &t0);

    let t0 = add_fp6(&t0, &t2);
    let t0 = mul_fp6(&t0, &t3);
    let y3 = add_fp6(&y3, &t0);

    let t2 = mul_fp6(self_y, self_z);
    let t2 = double_fp6(&t2);
    let t0 = mul_fp6(&t2, &t3);

    let x3 = sub_fp6(&x3, &t0);
    let z3 = mul_fp6(&t2, &t1);
    let z3 = double_fp6(&z3);

    let z3 = double_fp6(&z3);

    state[0..POINT_COORDINATE_WIDTH].copy_from_slice(&x3);
    state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH].copy_from_slice(&y3);
    state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH].copy_from_slice(&z3);
}

/// Compute the addition of the current point, stored as [X,Y,Z], with a given one.
/// Addition is computed as:
///
/// `X3 = (X1.Y2 + X2.Y1) (Y1.Y2 −(X1.Z2 + X2.Z1) − 3B.Z1.Z2)
///         − (Y1.Z2 + Y2.Z1) (X1.X2 + 3B(X1.Z2 + X2.Z1) − Z1.Z2)`
///
/// `Y3 = (3X1.X2 + Z1.Z2) (X1.X2 + 3B(X1.Z2 + X2.Z1) − Z1.Z2)
///         + (Y1.Y2 + (X1.Z2 + X2.Z1) + 3B.Z1.Z2) (Y1.Y2 −(X1.Z2 + X2.Z1) − 3B.Z1.Z2)`
///
/// `Z3 = (Y1.Z2 + Y2.Z1) (Y1.Y2 + (X1.Z2 + X2.Z1) + 3B.Z1.Z2)
///         + (X1.Y2 + X2.Y1) (3X1.X2 + Z1.Z2)`
#[inline(always)]
fn compute_add<E: FieldElement + From<BaseElement>>(state: &mut [E], point: &[E]) {
    let self_x = &state[0..POINT_COORDINATE_WIDTH];
    let self_y = &state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];
    let self_z = &state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH];

    let rhs_x = &point[0..POINT_COORDINATE_WIDTH];
    let rhs_y = &point[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];
    let rhs_z = &point[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH];

    let b3 = [
        E::from(B3[0]),
        E::from(B3[1]),
        E::from(B3[2]),
        E::from(B3[3]),
        E::from(B3[4]),
        E::from(B3[5]),
    ];

    let t0 = mul_fp6(self_x, rhs_x);
    let t1 = mul_fp6(self_y, rhs_y);
    let t2 = mul_fp6(self_z, rhs_z);

    let t3 = add_fp6(self_x, self_y);
    let t4 = add_fp6(rhs_x, rhs_y);
    let t3 = mul_fp6(&t3, &t4);

    let t4 = add_fp6(&t0, &t1);
    let t3 = sub_fp6(&t3, &t4);
    let t4 = add_fp6(self_x, self_z);

    let t5 = add_fp6(rhs_x, rhs_z);
    let t4 = mul_fp6(&t4, &t5);
    let t5 = add_fp6(&t0, &t2);

    let t4 = sub_fp6(&t4, &t5);
    let t5 = add_fp6(self_y, self_z);
    let x3 = add_fp6(rhs_y, rhs_z);

    let t5 = mul_fp6(&t5, &x3);
    let x3 = add_fp6(&t1, &t2);
    let t5 = sub_fp6(&t5, &x3);

    let x3 = mul_fp6(&b3, &t2);
    let z3 = add_fp6(&x3, &t4);

    let x3 = sub_fp6(&t1, &z3);
    let z3 = add_fp6(&t1, &z3);
    let y3 = mul_fp6(&x3, &z3);

    let t1 = double_fp6(&t0);
    let t1 = add_fp6(&t1, &t0);

    let t4 = mul_fp6(&b3, &t4);
    let t1 = add_fp6(&t1, &t2);
    let t2 = sub_fp6(&t0, &t2);

    let t4 = add_fp6(&t4, &t2);
    let t0 = mul_fp6(&t1, &t4);

    let y3 = add_fp6(&y3, &t0);
    let t0 = mul_fp6(&t5, &t4);
    let x3 = mul_fp6(&t3, &x3);

    let x3 = sub_fp6(&x3, &t0);
    let t0 = mul_fp6(&t3, &t1);
    let z3 = mul_fp6(&t5, &z3);

    let z3 = add_fp6(&z3, &t0);

    state[0..POINT_COORDINATE_WIDTH].copy_from_slice(&x3);
    state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH].copy_from_slice(&y3);
    state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH].copy_from_slice(&z3);
}

/// Compute the addition of the current point, stored as [X,Y] in affine coordinates, with another
/// point. It aditionals receives as input slope, which is assumed to be equal to (Y2 - Y1)/(X2 - X1).
/// Addition is computed as:
///
///  X3 =  slope * slope - X1 - X2
///  Y3 = slope * (X1 - X3) - Y1
#[inline(always)]
pub fn compute_add_affine_with_slope<E: FieldElement + From<BaseElement>>(state: &mut [E], point: &[E], slope: &[E]) {
    let x1 = &state[0..POINT_COORDINATE_WIDTH];
    let y1 = &state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];

    let x2 = &point[0..POINT_COORDINATE_WIDTH];
    
    let x3 = &mul_fp6(slope, slope);
    let x3 = &sub_fp6(x3, x1);
    let x3 = &sub_fp6(x3, x2);
    let x3 = &sub_fp6(x3, point);
    
    let y3 = &sub_fp6(x1, x3);
    let y3 = &mul_fp6(slope, y3);
    let y3 = &sub_fp6(y3, y1);
    state[0..POINT_COORDINATE_WIDTH].copy_from_slice(x3);
    state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH].copy_from_slice(y3);
}

/// Compute the addition of the current point, stored as [X,Y] in affine coordinates, with another
/// point.
pub fn compute_add_affine<E: FieldElement + From<BaseElement>>(state: &mut [E], point: &[E]) {
    let mut slope = [E::ZERO; POINT_COORDINATE_WIDTH];
    
    compute_slope(&mut slope, state, point);   
    compute_add_affine_with_slope(state, point, &slope);
}

/// Computes the slope required in the addition.
/// The slope is computed as:
///
///  slope = (Y2 - Y1)/(X2 - X1)
pub fn compute_slope<E: FieldElement + From<BaseElement>>(state: &mut[E], point1: &[E], point2: &[E]) {
    let x1 = &point1[0..POINT_COORDINATE_WIDTH];
    let y1 = &point1[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];

    let x2 = &point2[0..POINT_COORDINATE_WIDTH];
    let y2 = &point2[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];

    let numerator = sub_fp6(y2, y1);
    let mut denominator = sub_fp6(x2, x1);
    denominator = invert_fp6(&denominator);
    let slope = mul_fp6(&numerator, &denominator);
    state.copy_from_slice(&slope);

}

/// Compute the addition of the current point, stored as [X,Y,Z], with a given one
/// in affine coordinate (Z2 == 1).
/// Addition is computed as:
///
/// `X3 = (X1.Y2 + X2.Y1) (Y1.Y2 −(X1 + X2.Z1) − 3B.Z1)
///         − (Y1 + Y2.Z1) (X1.X2 + 3B(X1 + X2.Z1) − Z1)`
///
/// `Y3 = (3X1.X2 + Z1) (X1.X2 + 3B(X1 + X2.Z1) − Z1)
///         + (Y1.Y2 + (X1 + X2.Z1) + 3B.Z1) (Y1.Y2 −(X1 + X2.Z1) − 3B.Z1)`
///
/// `Z3 = (Y1 + Y2.Z1) (Y1.Y2 + (X1 + X2.Z1) + 3B.Z1)
///         + (X1.Y2 + X2.Y1) (3X1.X2 + Z1)`
#[inline(always)]
fn compute_add_mixed<E: FieldElement + From<BaseElement>>(state: &mut [E], point: &[E]) {
    let self_x = &state[0..POINT_COORDINATE_WIDTH];
    let self_y = &state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];
    let self_z = &state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH];

    let rhs_x = &point[0..POINT_COORDINATE_WIDTH];
    let rhs_y = &point[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH];

    let b3 = [
        E::from(B3[0]),
        E::from(B3[1]),
        E::from(B3[2]),
        E::from(B3[3]),
        E::from(B3[4]),
        E::from(B3[5]),
    ];

    let t0 = mul_fp6(self_x, rhs_x);
    let t1 = mul_fp6(self_y, rhs_y);
    let t3 = add_fp6(rhs_x, rhs_y);

    let t4 = add_fp6(self_x, self_y);
    let t3 = mul_fp6(&t3, &t4);
    let t4 = add_fp6(&t0, &t1);

    let t3 = sub_fp6(&t3, &t4);
    let t4 = mul_fp6(rhs_x, self_z);
    let t4 = add_fp6(&t4, self_x);

    let t5 = mul_fp6(rhs_y, self_z);
    let t5 = add_fp6(&t5, self_y);

    let x3 = mul_fp6(self_z, &b3);
    let z3 = add_fp6(&x3, &t4);
    let x3 = sub_fp6(&t1, &z3);

    let z3 = add_fp6(&t1, &z3);
    let y3 = mul_fp6(&x3, &z3);
    let t1 = double_fp6(&t0);

    let t1 = add_fp6(&t1, &t0);
    let t4 = mul_fp6(&t4, &b3);

    let t1 = add_fp6(&t1, self_z);
    let t2 = sub_fp6(&t0, self_z);

    let t4 = add_fp6(&t4, &t2);
    let t0 = mul_fp6(&t1, &t4);
    let y3 = add_fp6(&y3, &t0);

    let t0 = mul_fp6(&t5, &t4);
    let x3 = mul_fp6(&t3, &x3);
    let x3 = sub_fp6(&x3, &t0);

    let t0 = mul_fp6(&t3, &t1);
    let z3 = mul_fp6(&t5, &z3);
    let z3 = add_fp6(&z3, &t0);

    state[0..POINT_COORDINATE_WIDTH].copy_from_slice(&x3);
    state[POINT_COORDINATE_WIDTH..AFFINE_POINT_WIDTH].copy_from_slice(&y3);
    state[AFFINE_POINT_WIDTH..PROJECTIVE_POINT_WIDTH].copy_from_slice(&z3);
}

#[inline(always)]
pub(crate) fn square_fp2<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; 2] {
    let aa = a[0].square();
    let bb = a[1].square();

    let tmp = a[0].sub(a[1]);
    let tmp = tmp.square();

    let c0 = bb.double();
    let c0 = c0.add(aa);

    let c1 = bb.add(c0);
    let c1 = c1.sub(tmp);

    [c0, c1]
}

#[inline(always)]
pub(crate) fn mul_fp2<E: FieldElement + From<BaseElement>>(a: &[E], b: &[E]) -> [E; 2] {
    let aa = a[0].mul(b[0]);
    let bb = a[1].mul(b[1]);

    let tmp = a[0].sub(a[1]);
    let tmp2 = b[1].sub(b[0]);
    let tmp = tmp.mul(tmp2);

    let c0 = bb.double();
    let c0 = c0.add(aa);

    let c1 = bb.add(c0);
    let c1 = c1.add(tmp);

    [c0, c1]
}

#[inline(always)]
pub(crate) fn invert_fp2<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; 2] {
    let t = (a[0].square() + a[0].double() * a[1] - a[1].square().double()).inv();

    [(a[0] + a[1].double()) * t, -a[1] * t]
}

#[inline(always)]
pub(crate) fn add_fp2<E: FieldElement + From<BaseElement>>(a: &[E], b: &[E]) -> [E; 2] {
    [a[0].add(b[0]), a[1].add(b[1])]
}

#[inline(always)]
pub(crate) fn double_fp2<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; 2] {
    [a[0].double(), a[1].double()]
}

#[inline(always)]
pub(crate) fn sub_fp2<E: FieldElement + From<BaseElement>>(a: &[E], b: &[E]) -> [E; 2] {
    [a[0].sub(b[0]), a[1].sub(b[1])]
}

#[inline(always)]
pub(crate) fn neg_fp2<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; 2] {
    [a[0].neg(), a[1].neg()]
}

#[inline(always)]
pub(crate) fn square_fp6<E: FieldElement + From<BaseElement>>(
    a: &[E],
) -> [E; POINT_COORDINATE_WIDTH] {
    let self_c0 = &a[0..2];
    let self_c1 = &a[2..4];
    let self_c2 = &a[4..POINT_COORDINATE_WIDTH];

    let aa = square_fp2(self_c0);
    let bb = square_fp2(self_c1);
    let cc = square_fp2(self_c2);

    let ab_ab = add_fp2(self_c0, self_c1);
    let ab_ab = square_fp2(&ab_ab);

    let ac_ac = add_fp2(self_c0, self_c2);
    let ac_ac = square_fp2(&ac_ac);

    let bc_bc = add_fp2(self_c1, self_c2);
    let bc_bc = square_fp2(&bc_bc);

    let tmp = add_fp2(&aa, &bb);
    let tmp = add_fp2(&tmp, &cc);

    let c0 = sub_fp2(&tmp, &bc_bc);

    let c1 = sub_fp2(&ab_ab, &bc_bc);
    let c1 = sub_fp2(&c1, &aa);

    let c2 = sub_fp2(&ac_ac, &tmp);
    let c2 = sub_fp2(&c2, &cc);
    let t2 = add_fp2(&bb, &bb);
    let c2 = add_fp2(&c2, &t2);

    [c0[0], c0[1], c1[0], c1[1], c2[0], c2[1]]
}

#[inline(always)]
pub(crate) fn mul_fp6<E: FieldElement + From<BaseElement>>(
    a: &[E],
    b: &[E],
) -> [E; POINT_COORDINATE_WIDTH] {
    let self_c0 = &a[0..2];
    let self_c1 = &a[2..4];
    let self_c2 = &a[4..POINT_COORDINATE_WIDTH];

    let other_c0 = &b[0..2];
    let other_c1 = &b[2..4];
    let other_c2 = &b[4..POINT_COORDINATE_WIDTH];

    let aa = mul_fp2(self_c0, other_c0);
    let bb = mul_fp2(self_c1, other_c1);
    let cc = mul_fp2(self_c2, other_c2);

    let ab_ab = add_fp2(self_c0, self_c1);
    let tmp = add_fp2(other_c0, other_c1);
    let ab_ab = mul_fp2(&ab_ab, &tmp);

    let ac_ac = add_fp2(self_c0, self_c2);
    let tmp = add_fp2(other_c0, other_c2);
    let ac_ac = mul_fp2(&ac_ac, &tmp);

    let bc_bc = add_fp2(self_c1, self_c2);
    let tmp = add_fp2(other_c1, other_c2);
    let bc_bc = mul_fp2(&bc_bc, &tmp);

    let tmp = add_fp2(&aa, &bb);
    let tmp = add_fp2(&tmp, &cc);

    let c0 = sub_fp2(&tmp, &bc_bc);

    let c1 = sub_fp2(&ab_ab, &bc_bc);
    let c1 = sub_fp2(&c1, &aa);

    let c2 = sub_fp2(&ac_ac, &tmp);
    let c2 = sub_fp2(&c2, &cc);
    let t2 = add_fp2(&bb, &bb);
    let c2 = add_fp2(&c2, &t2);

    [c0[0], c0[1], c1[0], c1[1], c2[0], c2[1]]
}

#[inline(always)]
pub(crate) fn invert_fp6<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; POINT_COORDINATE_WIDTH] {
    let self_c0 = &a[0..2];
    let self_c1 = &a[2..4];
    let self_c2 = &a[4..POINT_COORDINATE_WIDTH];

    let c0_sq = square_fp2(self_c0);
    let c1_sq = square_fp2(self_c1);
    let c2_sq = square_fp2(self_c2);

    let t = mul_fp2(self_c0, &add_fp2(&c0_sq, &c1_sq));
    let t = sub_fp2(&t, &mul_fp2(self_c1, &c1_sq));
    let tmp = add_fp2(self_c0, &sub_fp2(self_c2, self_c1));
    let t = add_fp2(&t, &mul_fp2(&tmp, &c2_sq));
    let tmp = double_fp2(self_c0);
    let tmp = add_fp2(&tmp, self_c0);
    let tmp = mul_fp2(&tmp, self_c1);
    let tmp = sub_fp2(&double_fp2(&c0_sq), &tmp);
    let tmp = mul_fp2(&tmp, self_c2);
    let t = sub_fp2(&t, &tmp);

    let t = invert_fp2(&t);

    let c0 = add_fp2(&c0_sq, &c1_sq);
    let c0 = add_fp2(&c0, &c2_sq);
    let tmp = sub_fp2(&double_fp2(self_c0), self_c1);
    let tmp = mul_fp2(&tmp, self_c2);
    let c0 = sub_fp2(&c0, &tmp);
    let c0 = mul_fp2(&c0, &t);

    let c1 = mul_fp2(self_c0, self_c1);
    let c1 = add_fp2(&c1, &c2_sq);
    let c1 = neg_fp2(&c1);
    let c1 = mul_fp2(&c1, &t);

    let c2 = mul_fp2(self_c0, self_c2);
    let c2 = sub_fp2(&c1_sq, &c2);
    let c2 = add_fp2(&c2, &c2_sq);
    let c2 = mul_fp2(&c2, &t);

    [c0[0], c0[1], c1[0], c1[1], c2[0], c2[1]]
}

#[inline(always)]
pub(crate) fn add_fp6<E: FieldElement + From<BaseElement>>(
    a: &[E],
    b: &[E],
) -> [E; POINT_COORDINATE_WIDTH] {
    [
        a[0].add(b[0]),
        a[1].add(b[1]),
        a[2].add(b[2]),
        a[3].add(b[3]),
        a[4].add(b[4]),
        a[5].add(b[5]),
    ]
}

#[inline(always)]
pub(crate) fn double_fp6<E: FieldElement + From<BaseElement>>(
    a: &[E],
) -> [E; POINT_COORDINATE_WIDTH] {
    [
        a[0].double(),
        a[1].double(),
        a[2].double(),
        a[3].double(),
        a[4].double(),
        a[5].double(),
    ]
}

#[inline(always)]
pub(crate) fn sub_fp6<E: FieldElement + From<BaseElement>>(
    a: &[E],
    b: &[E],
) -> [E; POINT_COORDINATE_WIDTH] {
    [
        a[0].sub(b[0]),
        a[1].sub(b[1]),
        a[2].sub(b[2]),
        a[3].sub(b[3]),
        a[4].sub(b[4]),
        a[5].sub(b[5]),
    ]
}

#[inline(always)]
#[allow(unused)]
pub(crate) fn neg_fp6<E: FieldElement + From<BaseElement>>(a: &[E]) -> [E; POINT_COORDINATE_WIDTH] {
    [
        a[0].neg(),
        a[1].neg(),
        a[2].neg(),
        a[3].neg(),
        a[4].neg(),
        a[5].neg(),
    ]
}