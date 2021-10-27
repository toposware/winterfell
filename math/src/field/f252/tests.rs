// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::*;
use rand_utils::rand_value;

// Modulus
const M: BaseElement = BaseElement::from_raw_unchecked([
    0x0000000000000001,
    0x0000000000000000,
    0x0000000000000000,
    0x0800000000000011,
]);

const LARGEST: BaseElement = BaseElement::from_raw_unchecked([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0800000000000011,
]);

const TWO_POW_192: Repr = Repr([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000001,
]);

const TWO_POW_191: Repr = Repr([
    0x0000000000000000,
    0x0000000000000000,
    0x8000000000000000,
    0x0000000000000000,
]);

// BASIC ALGEBRA
// ================================================================================================

#[test]
fn test_equality() {
    assert_eq!(BaseElement::ZERO, BaseElement::ZERO);
    assert_eq!(BaseElement::ONE, BaseElement::ONE);

    assert!(BaseElement::ZERO != BaseElement::ONE);
}

#[test]
fn test_addition() {
    let mut tmp = LARGEST;
    tmp += &LARGEST;

    assert_eq!(
        tmp,
        BaseElement::from_raw_unchecked([
            0xffffffffffffffff,
            0xffffffffffffffff,
            0xffffffffffffffff,
            0x0800000000000010,
        ])
    );

    let mut tmp = LARGEST;
    tmp += &BaseElement::from_raw_unchecked([1, 0, 0, 0]);

    assert_eq!(tmp, BaseElement::ZERO);
}

#[test]
fn test_subtraction() {
    let mut tmp = LARGEST;
    tmp -= &LARGEST;

    assert_eq!(tmp, BaseElement::ZERO);

    let mut tmp = BaseElement::ZERO;
    tmp -= &LARGEST;

    let mut tmp2 = M;
    tmp2 -= &LARGEST;

    assert_eq!(tmp, tmp2);
}

#[test]
fn test_negation() {
    let tmp = -&LARGEST;

    assert_eq!(tmp, BaseElement::from_raw_unchecked([1, 0, 0, 0]));

    let tmp = -&BaseElement::ZERO;
    assert_eq!(tmp, BaseElement::ZERO);
    let tmp = -&BaseElement::from_raw_unchecked([1, 0, 0, 0]);
    assert_eq!(tmp, LARGEST);
}

#[test]
fn test_multiplication() {
    let mut cur = LARGEST;

    for _ in 0..100 {
        let mut tmp = cur;
        tmp *= &cur;

        let mut tmp2 = BaseElement::ZERO;
        for b in cur
            .to_bytes()
            .iter()
            .rev()
            .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1u8) == 1u8))
        {
            let tmp3 = tmp2;
            tmp2.add_assign(&tmp3);

            if b {
                tmp2.add_assign(&cur);
            }
        }

        assert_eq!(tmp, tmp2);

        cur.add_assign(&LARGEST);
    }
}

#[test]

fn test_inversion() {
    assert_eq!(BaseElement::ZERO.inv(), BaseElement::ZERO);
    assert_eq!(BaseElement::ONE.inv(), BaseElement::ONE);
    assert_eq!((-&BaseElement::ONE).inv(), -&BaseElement::ONE);

    let mut tmp: BaseElement = rand_value();

    for _ in 0..100 {
        let mut tmp2 = tmp.inv();
        tmp2.mul_assign(&tmp);

        assert_eq!(tmp2, BaseElement::ONE);

        tmp.add_assign(&tmp.clone());
    }
}

#[test]
fn test_squaring() {
    let mut cur = LARGEST;

    for _ in 0..100 {
        let mut tmp = cur;
        let pow2 = tmp.exp(Repr([2, 0, 0, 0]));
        tmp = tmp.square();

        let mut tmp2 = BaseElement::ZERO;
        for b in cur
            .to_bytes()
            .iter()
            .rev()
            .flat_map(|byte| (0..8).rev().map(move |i| ((byte >> i) & 1u8) == 1u8))
        {
            let tmp3 = tmp2;
            tmp2.add_assign(&tmp3);

            if b {
                tmp2.add_assign(&cur);
            }
        }

        assert_eq!(tmp, tmp2);
        assert_eq!(tmp, pow2);

        cur.add_assign(&LARGEST);
    }
}

#[test]
fn test_invert_is_pow() {
    let q_minus_2 = Repr([
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]);

    let mut r1 = BaseElement::ONE;
    let mut r2 = BaseElement::ONE;

    for _ in 0..100 {
        r1 = r1.inv();
        r2 = r2.exp(q_minus_2);

        assert_eq!(r1, r2);
        // Add R so we check something different next time around
        r1.add_assign(&BaseElement::ONE);
        r2 = r1;
    }
}

#[test]
fn test_bitand_repr() {
    {
        let a = <BaseElement as FieldElement>::Representation::from([7, 7, 7, 7]);
        let b = <BaseElement as FieldElement>::Representation::from([5, 5, 5, 5]);
        assert_eq!(a & b, Repr([5, 5, 5, 5]));
    }
    {
        let a = <BaseElement as FieldElement>::Representation::from([8, 8, 8, 8]);
        let b = <BaseElement as FieldElement>::Representation::from([5, 5, 5, 5]);
        assert_eq!(a & b, Repr([0, 0, 0, 0]));
    }
}

#[test]
fn test_shl_repr() {
    {
        // 2^3 x (2 + 2.2^64 + 2.2^128 + 2.2^192) = 16 + 16.2^64 + 16.2^128 + 16.2^192
        let mut a = <BaseElement as FieldElement>::Representation::from([2, 2, 2, 2]);
        a = a << 3;
        assert_eq!(a, Repr([16, 16, 16, 16]));
    }
    {
        // 2^64 x (2^64 - 1 + 0.2^64 + 0.2^128 + 0.2^192) = 0 + (2^64 - 1).2^64 + 0 + 0
        let mut a = <BaseElement as FieldElement>::Representation::from([u64::MAX, 0, 0, 0]);
        a = a << 64;
        assert_eq!(a, Repr([0, u64::MAX, 0, 0]));
    }
    {
        // 2^64 x (2^64 - 1 + 1.2^64 + 0.2^128 + 0.2^192) = 0 + (2^64 - 1).2^64 + 1.2^128
        let mut a = <BaseElement as FieldElement>::Representation::from([u64::MAX, 1, 0, 0]);
        a = a << 64;
        assert_eq!(a, Repr([0, u64::MAX, 1, 0]));
    }
}

#[test]
fn test_shr_repr() {
    {
        // (16 + 16.2^64 + 16.2^128 + 16.2^192) / 2^3 = 2 + 2.2^64 + 2.2^128 + 2.2^192
        let mut a = <BaseElement as FieldElement>::Representation::from([16, 16, 16, 16]);
        a = a >> 3;
        assert_eq!(a, Repr([2, 2, 2, 2]));
    }
    {
        // (2^64 - 2 + 1.2^64 + 0.2^128 + 0.2^192) / 2 = (2^65 - 2) / 2
        //                                             = (2^64 - 1) + 0 + 0 + 0
        let mut a = <BaseElement as FieldElement>::Representation::from([u64::MAX - 1, 1, 0, 0]);
        a = a >> 1;
        assert_eq!(a, Repr([u64::MAX, 0, 0, 0]));
    }
    {
        // (0 + (2^64 - 1).2^64 + 0.2^128 + 1.2^192) / 2^64 = (2^64 - 1) + 0.2^64 + 1.2^128 + 0
        let mut a = <BaseElement as FieldElement>::Representation::from([0, u64::MAX, 0, 1]);
        a = a >> 64;
        assert_eq!(a, Repr([u64::MAX, 0, 1, 0]));
    }
    {
        // (0 + (2^64 - 1).2^64 + (2^64 - 1).2^128 + 0.2^192) / 2^64 = (2^64 - 1) + (2^64 - 1).2^64 + 0 + 0
        let mut a = <BaseElement as FieldElement>::Representation::from([0, u64::MAX, u64::MAX, 0]);
        a = a >> 64;
        assert_eq!(a, Repr([u64::MAX, u64::MAX, 0, 0]));
    }
}

#[test]
fn test_conjugate() {
    let a: BaseElement = rand_value();
    let b = a.conjugate();
    assert_eq!(a, b);
}

// ROOTS OF UNITY
// ================================================================================================

#[test]
fn test_get_root_of_unity() {
    let root_192 = BaseElement::get_root_of_unity(192);
    assert_eq!(BaseElement::TWO_ADIC_ROOT_OF_UNITY, root_192);
    assert_eq!(BaseElement::ONE, root_192.exp(TWO_POW_192));

    let root_191 = BaseElement::get_root_of_unity(191);
    let expected = root_192.exp(Repr([2, 0, 0, 0]));
    assert_eq!(expected, root_191);
    assert_eq!(BaseElement::ONE, root_191.exp(TWO_POW_191));
}

#[test]
fn test_g_is_2_exp_192_root() {
    let g = BaseElement::TWO_ADIC_ROOT_OF_UNITY;
    assert_eq!(g.exp(TWO_POW_192), BaseElement::ONE);
}

// SERIALIZATION / DESERIALIZATION
// ================================================================================================

#[test]
fn test_to_bytes() {
    assert_eq!(
        BaseElement::ZERO.to_bytes(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );

    assert_eq!(
        BaseElement::ONE.to_bytes(),
        [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ]
    );

    assert_eq!(
        (-&BaseElement::ONE).to_bytes(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8,
        ]
    );
}

#[test]
fn test_from_bytes() {
    assert_eq!(
        BaseElement::from_bytes(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ])
        .unwrap(),
        BaseElement::ZERO
    );

    assert_eq!(
        BaseElement::from_bytes(&[
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0
        ])
        .unwrap(),
        BaseElement::ONE
    );

    // -1 should work
    assert_eq!(
        BaseElement::from_bytes(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8,
        ])
        .unwrap(),
        -BaseElement::ONE,
    );

    // M is invalid
    assert!(bool::from(
        BaseElement::from_bytes(&[
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8,
        ])
        .is_none()
    ));

    // Anything larger than the M is invalid
    assert!(bool::from(
        BaseElement::from_bytes(&[
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8,
        ])
        .is_none()
    ));
    assert!(bool::from(
        BaseElement::from_bytes(&[
            1, 0, 0, 0, 255, 255, 255, 255, 254, 91, 254, 255, 2, 164, 189, 83, 5, 216, 161, 9, 8,
            216, 58, 51, 72, 125, 157, 41, 83, 167, 237, 115
        ])
        .is_none()
    ));
    assert!(bool::from(
        BaseElement::from_bytes(&[
            153, 138, 183, 98, 118, 85, 192, 138, 212, 50, 253, 172, 212, 143, 5, 70, 43, 226, 210,
            217, 197, 56, 216, 63, 17, 0, 0, 0, 0, 0, 0, 8,
        ])
        .is_none()
    ));
}

#[test]
fn test_from_bytes_wide_negative_one() {
    assert_eq!(
        -&BaseElement::ONE,
        BaseElement::from_bytes_wide(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ])
    );
}

#[test]
fn test_from_bytes_wide_maximum() {
    assert_eq!(
        BaseElement::from_raw_unchecked([
            0xcc7177d1406df1ae,
            0x7545706677ffcc06,
            0xf47d84f836300018,
            0x038e5f79873c0c8d,
        ]),
        BaseElement::from_bytes_wide(&[0xff; 64])
    );
}

// INITIALIZATION
// ================================================================================================

#[test]
fn test_zeroed_vector() {
    let result = BaseElement::zeroed_vector(4);
    assert_eq!(4, result.len());
    for element in result.into_iter() {
        assert_eq!(BaseElement::ZERO, element);
    }
}
