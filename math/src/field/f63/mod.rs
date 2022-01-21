// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! An implementation of a 63-bit STARK-friendly prime field with modulus 2^62 + 2^56 + 2^55 + 1.
//!
//! Operations in this field are implemented using Montgomery reduction.

use super::{
    traits::{FieldElement, StarkField},
    ExtensibleField,
};
use cheetah::group::ff::Field;
use cheetah::Fp as BaseElementInner;
use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    slice,
};
use rand_core::RngCore;

use utils::{
    collections::Vec, string::ToString, AsBytes, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Randomizable, Serializable,
};

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

/// Field modulus = 2^62 + 2^56 + 2^55 + 1
const M: u64 = 4719772409484279809;

// 2^55 root of unity
const G: u64 = 90479342105353296;

// Number of bytes needed to represent field element
const ELEMENT_BYTES: usize = core::mem::size_of::<u64>();

// FIELD ELEMENT
// ================================================================================================

/// Represents a base field element.
///
/// Internal values are stored in Montgomery form.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct BaseElement(pub(crate) BaseElementInner);

impl Deref for BaseElement {
    type Target = BaseElementInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BaseElement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for BaseElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tmp = self.to_bytes();
        write!(f, "0x")?;
        for &b in tmp.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl Display for BaseElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl BaseElement {
    /// Creates a new field element from a u64 value.
    /// The value is converted to Montgomery form by computing
    /// (a.R^0 * R^2) / R = a.R
    pub const fn new(value: u64) -> Self {
        BaseElement(BaseElementInner::new(value))
    }

    pub const fn output_internal(&self) -> BaseElementInner {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn add(&self, rhs: &Self) -> Self {
        BaseElement(self.0.add(&rhs.0))
    }

    #[inline]
    #[must_use]
    pub fn sub(&self, rhs: &Self) -> Self {
        BaseElement(self.0.sub(&rhs.0))
    }

    #[inline]
    #[must_use]
    pub fn neg(&self) -> Self {
        BaseElement(self.0.neg())
    }

    #[inline]
    #[must_use]
    pub fn mul(&self, rhs: &Self) -> Self {
        BaseElement(self.0.mul(&rhs.0))
    }

    #[inline]
    #[must_use]
    pub fn square(&self) -> Self {
        BaseElement(self.0.square())
    }

    #[inline]
    #[must_use]
    pub fn double(&self) -> Self {
        BaseElement(self.0.double())
    }

    pub fn from_bytes(bytes: &[u8; 8]) -> Option<Self> {
        let tmp = BaseElementInner::from_bytes(bytes);
        if bool::from(tmp.is_none()) {
            None
        } else {
            Some(BaseElement(tmp.unwrap()))
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_bytes()
    }

    /// Returns whether or not this element is strictly lexicographically
    /// larger than its negation.
    pub fn lexicographically_largest(&self) -> bool {
        bool::from(self.0.lexicographically_largest())
    }

    /// Constructs an element of `BaseElement` without checking that it is
    /// canonical.
    #[must_use]
    pub const fn from_raw_unchecked(v: u64) -> Self {
        BaseElement(BaseElementInner::from_raw_unchecked(v))
    }

    /// Outputs the raw underlying u64 limb without Montgomery reduction
    pub const fn output_unreduced_limbs(&self) -> <Self as FieldElement>::Representation {
        self.0.output_unreduced_limbs()
    }

    /// Computes the square root of this element, if it exists.
    pub fn sqrt(&self) -> Option<Self> {
        let tmp = self.0.sqrt();
        if bool::from(tmp.is_none()) {
            None
        } else {
            Some(BaseElement(tmp.unwrap()))
        }
    }

    /// Generates a random field element
    #[must_use]
    pub fn random(mut rng: impl RngCore) -> Self {
        BaseElement(BaseElementInner::random(&mut rng))
    }
}

impl FieldElement for BaseElement {
    type Representation = u64;

    type BaseField = Self;

    const ZERO: Self = BaseElement(BaseElementInner::zero());
    const ONE: Self = BaseElement(BaseElementInner::one());

    const ELEMENT_BYTES: usize = ELEMENT_BYTES;

    const IS_CANONICAL: bool = false;

    fn inv(self) -> Self {
        BaseElement(self.invert().unwrap_or(BaseElementInner::zero()))
    }

    fn conjugate(&self) -> Self {
        BaseElement(self.0)
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        let p = elements.as_ptr();
        let len = elements.len() * Self::ELEMENT_BYTES;
        unsafe { slice::from_raw_parts(p as *const u8, len) }
    }

    unsafe fn bytes_as_elements(bytes: &[u8]) -> Result<&[Self], DeserializationError> {
        if bytes.len() % Self::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(format!(
                "number of bytes ({}) does not divide into whole number of field elements",
                bytes.len(),
            )));
        }

        let p = bytes.as_ptr();
        let len = bytes.len() / Self::ELEMENT_BYTES;

        if (p as usize) % mem::align_of::<u64>() != 0 {
            return Err(DeserializationError::InvalidValue(
                "slice memory alignment is not valid for this field element type".to_string(),
            ));
        }

        Ok(slice::from_raw_parts(p as *const Self, len))
    }

    fn zeroed_vector(n: usize) -> Vec<Self> {
        // this uses a specialized vector initialization code which requests zero-filled memory
        // from the OS; unfortunately, this works only for built-in types and we can't use
        // Self::ZERO here as much less efficient initialization procedure will be invoked.
        debug_assert_eq!(Self::ELEMENT_BYTES, mem::size_of::<u64>());
        let result = vec![0u64; n];

        // translate a zero-filled vector of u64s into a vector of base field elements
        let mut v = core::mem::ManuallyDrop::new(result);
        let p = v.as_mut_ptr();
        let len = v.len();
        let cap = v.capacity();
        unsafe { Vec::from_raw_parts(p as *mut Self, len, cap) }
    }

    fn as_base_elements(elements: &[Self]) -> &[Self::BaseField] {
        elements
    }
}

impl StarkField for BaseElement {
    /// sage: MODULUS = 2^62 + 2^56 + 2^55 + 1 \
    /// sage: GF(MODULUS).is_prime_field() \
    /// True \
    /// sage: GF(MODULUS).order() \
    /// 4719772409484279809
    const MODULUS: Self::Representation = M;
    const MODULUS_BITS: u32 = 63;

    /// sage: GF(MODULUS).primitive_element() \
    /// 3
    const GENERATOR: Self = BaseElement::new(3);

    /// sage: is_odd((MODULUS - 1) / 2^55) \
    /// True
    const TWO_ADICITY: u32 = 55;

    /// sage: k = (MODULUS - 1) / 2^55 \
    /// sage: GF(MODULUS).primitive_element()^k \
    /// 90479342105353296
    const TWO_ADIC_ROOT_OF_UNITY: Self = BaseElement::new(G);

    fn get_root_of_unity(n: u32) -> Self {
        BaseElement(BaseElementInner::get_root_of_unity_vartime(n))
    }

    fn get_modulus_le_bytes() -> Vec<u8> {
        Self::MODULUS.to_le_bytes().to_vec()
    }

    fn to_repr(&self) -> Self::Representation {
        self.output_reduced_limbs()
    }
}

impl Randomizable for BaseElement {
    const VALUE_SIZE: usize = Self::ELEMENT_BYTES;

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(bytes).ok()
    }
}

// OVERLOADED OPERATORS
// ================================================================================================

impl<'a> Neg for &'a BaseElement {
    type Output = BaseElement;

    #[inline]
    fn neg(self) -> BaseElement {
        self.neg()
    }
}

impl Neg for BaseElement {
    type Output = BaseElement;

    #[inline]
    fn neg(self) -> BaseElement {
        -&self
    }
}

impl<'a, 'b> Sub<&'b BaseElement> for &'a BaseElement {
    type Output = BaseElement;

    #[inline]
    fn sub(self, rhs: &'b BaseElement) -> BaseElement {
        self.sub(rhs)
    }
}

impl<'a, 'b> Add<&'b BaseElement> for &'a BaseElement {
    type Output = BaseElement;

    #[inline]
    fn add(self, rhs: &'b BaseElement) -> BaseElement {
        self.add(rhs)
    }
}

impl Sub<BaseElement> for BaseElement {
    type Output = BaseElement;

    #[inline]
    fn sub(self, rhs: BaseElement) -> BaseElement {
        BaseElement(self.0 - rhs.0)
    }
}

impl Add<BaseElement> for BaseElement {
    type Output = BaseElement;

    #[inline]
    fn add(self, rhs: BaseElement) -> BaseElement {
        BaseElement(self.0 + rhs.0)
    }
}

impl SubAssign<BaseElement> for BaseElement {
    #[inline]
    fn sub_assign(&mut self, rhs: BaseElement) {
        *self = *self - rhs;
    }
}

impl AddAssign<BaseElement> for BaseElement {
    #[inline]
    fn add_assign(&mut self, rhs: BaseElement) {
        *self = *self + rhs;
    }
}

impl<'b> SubAssign<&'b BaseElement> for BaseElement {
    #[inline]
    fn sub_assign(&mut self, rhs: &'b BaseElement) {
        *self = &*self - rhs;
    }
}

impl<'b> AddAssign<&'b BaseElement> for BaseElement {
    #[inline]
    fn add_assign(&mut self, rhs: &'b BaseElement) {
        *self = &*self + rhs;
    }
}

impl Mul<BaseElement> for BaseElement {
    type Output = BaseElement;

    #[inline]
    fn mul(self, rhs: BaseElement) -> BaseElement {
        BaseElement(self.0 * rhs.0)
    }
}

impl<'a, 'b> Mul<&'b BaseElement> for &'a BaseElement {
    type Output = BaseElement;

    #[inline]
    fn mul(self, rhs: &'b BaseElement) -> BaseElement {
        self.mul(rhs)
    }
}

impl MulAssign<BaseElement> for BaseElement {
    #[inline]
    fn mul_assign(&mut self, rhs: BaseElement) {
        *self = *self * rhs;
    }
}

impl<'b> MulAssign<&'b BaseElement> for BaseElement {
    #[inline]
    fn mul_assign(&mut self, rhs: &'b BaseElement) {
        *self = &*self * rhs;
    }
}

impl Div for BaseElement {
    type Output = Self;

    fn div(self, rhs: Self) -> BaseElement {
        self.mul(rhs.inv())
    }
}

impl DivAssign for BaseElement {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

// QUADRATIC EXTENSION
// ================================================================================================

/// Defines a quadratic extension of the base field over an irreducible polynomial x<sup>2</sup> -
/// 2*x - 2. Thus, an extension element is defined as α + β * φ, where φ is a root of this polynomial,
/// and α and β are base field elements.
impl ExtensibleField<2> for BaseElement {
    #[inline(always)]
    fn mul(a: [Self; 2], b: [Self; 2]) -> [Self; 2] {
        let a0b0 = a[0] * b[0];
        let a1b1 = a[1] * b[1];

        let t = (a[0] - a[1]) * (b[1] - b[0]);

        let res0 = a0b0 + a1b1.double();
        [res0, a1b1 + res0 + t]
    }

    #[inline(always)]
    fn frobenius(x: [Self; 2]) -> [Self; 2] {
        [x[0] + x[1].double(), -x[1]]
    }
}

// CUBIC EXTENSION
// ================================================================================================

/// Defines a cubic extension of the base field over an irreducible polynomial x<sup>3</sup> +
/// x + 1. Thus, an extension element is defined as α + β * φ + γ * φ^2, where φ is a root of this
/// polynomial, and α, β and γ are base field elements.
impl ExtensibleField<3> for BaseElement {
    #[inline(always)]
    fn mul(a: [Self; 3], b: [Self; 3]) -> [Self; 3] {
        // performs multiplication in the extension field using 6 multiplications,
        // 10 additions, and 4 subtractions in the base field.
        let a0b0 = a[0] * b[0];
        let a1b1 = a[1] * b[1];
        let a2b2 = a[2] * b[2];

        let a0b0_a0b1_a1b0_a1b1 = (a[0] + a[1]) * (b[0] + b[1]);
        let a0b0_a0b2_a2b0_a2b2 = (a[0] + a[2]) * (b[0] + b[2]);
        let a1b1_a1b2_a2b1_a2b2 = (a[1] + a[2]) * (b[1] + b[2]);

        let a0b0_a1b1_a2b2 = a0b0 + a1b1 + a2b2;

        let res0 = a0b0_a1b1_a2b2 - a1b1_a1b2_a2b1_a2b2;
        let res1 = a0b0_a0b1_a1b0_a1b1 - a1b1_a1b2_a2b1_a2b2 - a0b0;
        let res2 = a0b0_a0b2_a2b0_a2b2 - a0b0_a1b1_a2b2 - a2b2 + a1b1.double();

        [res0, res1, res2]
    }

    #[inline(always)]
    fn frobenius(x: [Self; 3]) -> [Self; 3] {
        // coefficients were computed using SageMath
        [
            x[0] + BaseElement::new(3748426544840615980) * x[1]
                + BaseElement::new(902867407776644160) * x[2],
            BaseElement::new(3365471297819313567) * x[1]
                + BaseElement::new(1874213272420307990) * x[2],
            BaseElement::new(902867407776644161) * x[1]
                + BaseElement::new(1354301111664966241) * x[2],
        ]
    }
}

// TYPE CONVERSIONS
// ================================================================================================

impl From<BaseElementInner> for BaseElement {
    /// Converts a 128-bit value into a field element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently performed.
    fn from(value: BaseElementInner) -> Self {
        BaseElement(value)
    }
}

impl From<u128> for BaseElement {
    /// Converts a 128-bit value into a field element.
    fn from(value: u128) -> Self {
        BaseElement(BaseElementInner::from_bytes_wide(&value.to_le_bytes()))
    }
}

impl From<u64> for BaseElement {
    /// Converts a 64-bit value into a field element.
    fn from(value: u64) -> Self {
        BaseElement(BaseElementInner::from(value))
    }
}

impl From<u32> for BaseElement {
    /// Converts a 32-bit value into a field element.
    fn from(value: u32) -> Self {
        BaseElement(BaseElementInner::from(value))
    }
}

impl From<u16> for BaseElement {
    /// Converts a 16-bit value into a field element.
    fn from(value: u16) -> Self {
        BaseElement(BaseElementInner::from(value))
    }
}

impl From<u8> for BaseElement {
    /// Converts an 8-bit value into a field element.
    fn from(value: u8) -> Self {
        BaseElement(BaseElementInner::from(value))
    }
}

impl From<[u8; 8]> for BaseElement {
    /// Converts the value encoded in an array of 32 bytes into a field element. The bytes
    /// are assumed to be in little-endian byte order. If the value is greater than or equal
    /// to the field modulus, modular reduction is silently performed.
    fn from(bytes: [u8; 8]) -> Self {
        Self::from_bytes(&bytes).unwrap_or(Self::ZERO)
    }
}

impl From<&[u8; 8]> for BaseElement {
    /// Converts the value encoded in an array of 32 bytes into a field element. The bytes
    /// are assumed to be in little-endian byte order. If the value is greater than or equal
    /// to the field modulus, modular reduction is silently performed.
    fn from(bytes: &[u8; 8]) -> Self {
        Self::from_bytes(bytes).unwrap_or(Self::ZERO)
    }
}

impl<'a> TryFrom<&'a [u8]> for BaseElement {
    type Error = DeserializationError;

    /// Converts a slice of bytes into a field element; returns error if the value encoded in bytes
    /// is not a valid field element. The bytes are assumed to be in little-endian byte order.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }
        if bytes.len() > ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }

        BaseElement::from_bytes(bytes.try_into().unwrap()).ok_or_else(|| {
            DeserializationError::InvalidValue(
                "invalid field element: value is greater than or equal to the field modulus"
                    .to_string(),
            )
        })
    }
}

impl AsBytes for BaseElement {
    fn as_bytes(&self) -> &[u8] {
        // TODO: take endianness into account
        let self_ptr: *const BaseElement = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, BaseElement::ELEMENT_BYTES) }
    }
}

// SERIALIZATION / DESERIALIZATION
// ------------------------------------------------------------------------------------------------

impl Serializable for BaseElement {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.to_bytes());
    }
}

impl Deserializable for BaseElement {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let bytes = source.read_u8_array()?;
        if bytes.len() < ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }
        if bytes.len() > ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }

        Ok(BaseElement::from_bytes(&bytes).unwrap_or(BaseElement::ZERO))
    }
}
