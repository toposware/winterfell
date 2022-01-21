// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::{BaseElement, DeserializationError, FieldElement, StarkField};
use crate::field::{CubeExtension, QuadExtension};
use core::convert::TryFrom;
use num_bigint::BigUint;
use proptest::prelude::*;
use rand_utils::rand_value;

// MANUAL TESTS
// ================================================================================================

#[test]
fn add() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(r, r + BaseElement::ZERO);

    // test addition within bounds
    assert_eq!(
        BaseElement::from(5u8),
        BaseElement::from(2u8) + BaseElement::from(3u8)
    );

    // test overflow
    let t = BaseElement::from(BaseElement::MODULUS - 1);
    assert_eq!(BaseElement::ZERO, t + BaseElement::ONE);
    assert_eq!(BaseElement::ONE, t + BaseElement::from(2u8));
}

#[test]
fn sub() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(r, r - BaseElement::ZERO);

    // test subtraction within bounds
    assert_eq!(
        BaseElement::from(2u8),
        BaseElement::from(5u8) - BaseElement::from(3u8)
    );

    // test underflow
    let expected = BaseElement::from(BaseElement::MODULUS - 2);
    assert_eq!(expected, BaseElement::from(3u8) - BaseElement::from(5u8));
}

#[test]
fn mul() {
    // identity
    let r: BaseElement = rand_value();
    assert_eq!(BaseElement::ZERO, r * BaseElement::ZERO);
    assert_eq!(r, r * BaseElement::ONE);

    // test multiplication within bounds
    assert_eq!(
        BaseElement::from(15u8),
        BaseElement::from(5u8) * BaseElement::from(3u8)
    );

    // test overflow
    let m = BaseElement::MODULUS;
    let t = BaseElement::from(m - 1);
    assert_eq!(BaseElement::ONE, t * t);
    assert_eq!(BaseElement::from(m - 2), t * BaseElement::from(2u8));
    assert_eq!(BaseElement::from(m - 4), t * BaseElement::from(4u8));

    let t = (m + 1) / 2;
    assert_eq!(
        BaseElement::ONE,
        BaseElement::from(t) * BaseElement::from(2u8)
    );
}

#[test]
fn exp() {
    let a = BaseElement::ZERO;
    assert_eq!(a.exp(0), BaseElement::ONE);
    assert_eq!(a.exp(1), BaseElement::ZERO);

    let a = BaseElement::ONE;
    assert_eq!(a.exp(0), BaseElement::ONE);
    assert_eq!(a.exp(1), BaseElement::ONE);
    assert_eq!(a.exp(3), BaseElement::ONE);

    let a: BaseElement = rand_value();
    assert_eq!(a.exp(3), a * a * a);
}

#[test]
fn inv() {
    // identity
    assert_eq!(BaseElement::ONE, BaseElement::inv(BaseElement::ONE));
    assert_eq!(BaseElement::ZERO, BaseElement::inv(BaseElement::ZERO));
}

#[test]
fn element_as_int() {
    let v = u64::MAX;
    let e = BaseElement::new(v);
    assert_eq!(v % super::M, e.to_repr());
}

// QUADRATIC EXTENSION
// ------------------------------------------------------------------------------------------------
#[test]
fn quad_mul() {
    // identity
    let r: QuadExtension<BaseElement> = rand_value();
    assert_eq!(
        <QuadExtension<BaseElement>>::ZERO,
        r * <QuadExtension<BaseElement>>::ZERO
    );
    assert_eq!(r, r * <QuadExtension<BaseElement>>::ONE);

    // test multiplication within bounds
    let a = <QuadExtension<BaseElement>>::new(BaseElement::new(15), BaseElement::new(22));
    let b = <QuadExtension<BaseElement>>::new(BaseElement::new(20), BaseElement::new(22));
    let expected =
        <QuadExtension<BaseElement>>::new(BaseElement::new(1268), BaseElement::new(1738));
    assert_eq!(expected, a * b);

    // test multiplication with overflow
    let a = <QuadExtension<BaseElement>>::new(
        BaseElement::new(909293122448838652),
        BaseElement::new(477277787758322091),
    );
    let b = <QuadExtension<BaseElement>>::new(
        BaseElement::new(3542703669471729099),
        BaseElement::new(1163739192097758270),
    );
    let expected = <QuadExtension<BaseElement>>::new(
        BaseElement::new(2134274952469056161),
        BaseElement::new(2186070506318879173),
    );
    assert_eq!(expected, a * b);
}

// CUBIC EXTENSION
// ------------------------------------------------------------------------------------------------
#[test]
fn cube_mul() {
    // identity
    let r: CubeExtension<BaseElement> = rand_value();
    assert_eq!(
        <CubeExtension<BaseElement>>::ZERO,
        r * <CubeExtension<BaseElement>>::ZERO
    );
    assert_eq!(r, r * <CubeExtension<BaseElement>>::ONE);

    // test multiplication within bounds
    let a = <CubeExtension<BaseElement>>::new(
        BaseElement::new(15),
        BaseElement::new(22),
        BaseElement::new(8),
    );
    let b = <CubeExtension<BaseElement>>::new(
        BaseElement::new(20),
        BaseElement::new(22),
        BaseElement::new(6),
    );
    let expected = <CubeExtension<BaseElement>>::new(
        BaseElement::new(4719772409484279801),
        BaseElement::new(414),
        BaseElement::new(686),
    );
    assert_eq!(expected, a * b);

    // test multiplication with overflow
    let a = <CubeExtension<BaseElement>>::new(
        BaseElement::new(2517249252820153010),
        BaseElement::new(2670313043742480964),
        BaseElement::new(1034485803185789129),
    );
    let b = <CubeExtension<BaseElement>>::new(
        BaseElement::new(3899352077385304145),
        BaseElement::new(3597623373506293891),
        BaseElement::new(2869585688428194301),
    );
    let expected = <CubeExtension<BaseElement>>::new(
        BaseElement::new(3902260085483020565),
        BaseElement::new(4183873908669523882),
        BaseElement::new(2196229189664165138),
    );
    assert_eq!(expected, a * b);
}

// ROOTS OF UNITY
// ------------------------------------------------------------------------------------------------

#[test]
fn get_root_of_unity() {
    let root_55 = BaseElement::get_root_of_unity(55);
    assert_eq!(BaseElement::TWO_ADIC_ROOT_OF_UNITY, root_55);
    assert_eq!(BaseElement::ONE, root_55.exp(1u64 << 55));

    let root_54 = BaseElement::get_root_of_unity(54);
    let expected = root_55.exp(2);
    assert_eq!(expected, root_54);
    assert_eq!(BaseElement::ONE, root_54.exp(1u64 << 54));
}

// SERIALIZATION AND DESERIALIZATION
// ------------------------------------------------------------------------------------------------

#[test]
fn from_u128() {
    let v = u128::MAX;
    // e = R3 - R in Montgomery form
    //   = R2 - 1 in reduced form
    let e = BaseElement::from(v);
    // R2 - 1 = 3635333122111952145
    assert_eq!(3635333122111952145, e.to_repr());
}

#[test]
fn try_from_slice() {
    let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_ok());
    assert_eq!(1, result.unwrap().to_repr());

    let bytes = vec![1, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());

    let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());

    let bytes = vec![255, 255, 255, 255, 255, 255, 255, 255];
    let result = BaseElement::try_from(bytes.as_slice());
    assert!(result.is_err());
}

#[test]
fn elements_as_bytes() {
    let source = vec![
        BaseElement::new(1),
        BaseElement::new(2),
        BaseElement::new(3),
        BaseElement::new(4),
    ];

    let mut expected = vec![];
    expected.extend_from_slice(&source[0].output_unreduced_limbs().to_le_bytes());
    expected.extend_from_slice(&source[1].output_unreduced_limbs().to_le_bytes());
    expected.extend_from_slice(&source[2].output_unreduced_limbs().to_le_bytes());
    expected.extend_from_slice(&source[3].output_unreduced_limbs().to_le_bytes());

    assert_eq!(expected, BaseElement::elements_as_bytes(&source));
}

#[test]
fn bytes_as_elements() {
    let elements = vec![
        BaseElement::new(1),
        BaseElement::new(2),
        BaseElement::new(3),
        BaseElement::new(4),
    ];

    let mut bytes = vec![];
    bytes.extend_from_slice(&elements[0].output_unreduced_limbs().to_le_bytes());
    bytes.extend_from_slice(&elements[1].output_unreduced_limbs().to_le_bytes());
    bytes.extend_from_slice(&elements[2].output_unreduced_limbs().to_le_bytes());
    bytes.extend_from_slice(&elements[3].output_unreduced_limbs().to_le_bytes());
    bytes.extend_from_slice(&BaseElement::new(5).output_unreduced_limbs().to_le_bytes());

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[..32]) };
    assert!(result.is_ok());
    assert_eq!(elements, result.unwrap());

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[..33]) };
    assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));

    let result = unsafe { BaseElement::bytes_as_elements(&bytes[1..33]) };
    assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));
}

// INITIALIZATION
// ------------------------------------------------------------------------------------------------

#[test]
fn zeroed_vector() {
    let result = BaseElement::zeroed_vector(4);
    assert_eq!(4, result.len());
    for element in result.into_iter() {
        assert_eq!(BaseElement::ZERO, element);
    }
}

// RANDOMIZED TESTS
// ================================================================================================

proptest! {

    #[test]
    fn add_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 + v2;

        let expected = (a % super::M + b % super::M) % super::M;
        prop_assert_eq!(expected, result.to_repr());
    }

    #[test]
    fn sub_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 - v2;

        let a = a % super::M;
        let b = b % super::M;
        let expected = if a < b { super::M - b + a } else { a - b };

        prop_assert_eq!(expected, result.to_repr());
    }

    #[test]
    fn mul_proptest(a in any::<u64>(), b in any::<u64>()) {
        let v1 = BaseElement::from(a);
        let v2 = BaseElement::from(b);
        let result = v1 * v2;

        let expected = (((a as u128) * (b as u128)) % super::M as u128) as u64;
        prop_assert_eq!(expected, result.to_repr());
    }

    #[test]
    fn exp_proptest(a in any::<u64>(), b in any::<u64>()) {
        let result = BaseElement::from(a).exp(b);

        let b = BigUint::from(b);
        let m = BigUint::from(super::M);
        let expected = BigUint::from(a).modpow(&b, &m).to_u64_digits()[0];
        prop_assert_eq!(expected, result.to_repr());
    }

    #[test]
    fn inv_proptest(a in any::<u64>()) {
        let a = BaseElement::from(a);
        let b = a.inv();

        let expected = if a == BaseElement::ZERO { BaseElement::ZERO } else { BaseElement::ONE };
        prop_assert_eq!(expected, a * b);
    }

    #[test]
    fn element_as_int_proptest(a in any::<u64>()) {
        let e = BaseElement::new(a);
        prop_assert_eq!(a % super::M, e.to_repr());
    }

    // QUADRATIC EXTENSION
    // --------------------------------------------------------------------------------------------
    #[test]
    fn quad_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>()) {
        let a = QuadExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1));
        let b = a.inv();

        let expected = if a == QuadExtension::<BaseElement>::ZERO {
            QuadExtension::<BaseElement>::ZERO
        } else {
            QuadExtension::<BaseElement>::ONE
        };
        prop_assert_eq!(expected, a * b);
    }

    // CUBIC EXTENSION
    // --------------------------------------------------------------------------------------------
    #[test]
    fn cube_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>(), a2 in any::<u64>()) {
        let a = CubeExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1), BaseElement::from(a2));
        let b = a.inv();

        let expected = if a == CubeExtension::<BaseElement>::ZERO {
            CubeExtension::<BaseElement>::ZERO
        } else {
            CubeExtension::<BaseElement>::ONE
        };
        prop_assert_eq!(expected, a * b);
    }
}
