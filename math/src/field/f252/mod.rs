// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! An implementation of a 252-bit STARK-friendly prime field with modulus 2^251 + 17 * 2^192 + 1.
//!
//! Operations in this field are implemented using Barret reduction and are stored in their
//! canonical form using `[u64; 4]` as the backing type.
//!
//! Implementation is clearly not optimal!

use super::traits::{ExtensibleField, FieldElement, StarkField};
use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug, Display, Formatter},
    mem,
    ops::{
        Add, AddAssign, BitAnd, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Shl, Shr,
        ShrAssign, Sub, SubAssign,
    },
    slice,
};
use stark_curve::FieldElement as BaseElementInner;
use utils::{
    collections::Vec, string::ToString, AsBytes, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Randomizable, Serializable,
};

/// Compute (a << b) + carry, returning the result and the new carry over.
#[inline(always)]
const fn shl32_with_carry(a: u64, b: u32, carry: u64) -> (u64, u64) {
    let ret = ((a as u128) << (b as u128)) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}

/// Compute (a >> b) + carry, returning the result and the new carry over.
#[inline(always)]
const fn shr32_with_carry(a: u64, b: u32, carry: u64) -> (u64, u64) {
    let ret = ((a as u128) << 64) >> (b as u128);
    (((ret >> 64) + (carry as u128)) as u64, ret as u64)
}

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Default)]
pub struct Repr(pub [u64; 4]);

impl From<u32> for Repr {
    fn from(value: u32) -> Self {
        Repr([value as u64, 0, 0, 0])
    }
}

impl From<u64> for Repr {
    fn from(value: u64) -> Self {
        Repr([value, 0, 0, 0])
    }
}

impl From<[u64; 4]> for Repr {
    fn from(value: [u64; 4]) -> Self {
        Repr(value)
    }
}

impl Deref for Repr {
    type Target = [u64; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BitAnd for Repr {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl Shl<u32> for Repr {
    type Output = Self;

    /// Performs a left shift on a value represented as 4 u64 limbs,
    /// ordered in little-endian.
    fn shl(self, rhs: u32) -> Self::Output {
        if rhs > 255 {
            return Self([0, 0, 0, 0]);
        }
        let mut rhs = rhs % 256;
        let mut array = self.0;
        while rhs > 0 {
            let shift = if rhs > 64 { 64 } else { rhs % 65 };
            let (res0, carry) = shl32_with_carry(array[0], shift, 0);
            let (res1, carry) = shl32_with_carry(array[1], shift, carry);
            let (res2, carry) = shl32_with_carry(array[2], shift, carry);
            let (res3, _carry) = shl32_with_carry(array[3], shift, carry);
            array = [res0, res1, res2, res3];
            rhs = rhs.saturating_sub(shift);
        }

        Self(array)
    }
}

impl Shr<u32> for Repr {
    type Output = Self;

    /// Performs a right shift on a value represented as 4 u64 limbs,
    /// ordered in little-endian.
    fn shr(self, rhs: u32) -> Self::Output {
        if rhs > 255 {
            return Self([0, 0, 0, 0]);
        }
        let mut rhs = rhs % 256;
        let mut array = self.0;
        while rhs > 0 {
            let shift = if rhs > 64 { 64 } else { rhs % 65 };
            let (res3, carry) = shr32_with_carry(array[3], shift, 0);
            let (res2, carry) = shr32_with_carry(array[2], shift, carry);
            let (res1, carry) = shr32_with_carry(array[1], shift, carry);
            let (res0, _carry) = shr32_with_carry(array[0], shift, carry);
            array = [res0, res1, res2, res3];
            rhs = rhs.saturating_sub(shift);
        }

        Self(array)
    }
}

impl ShrAssign for Repr {
    fn shr_assign(&mut self, rhs: Self) {
        *self = *self >> (rhs[0] as u32);
    }
}

// CONSTANTS
// ================================================================================================

// 2^192 root of unity = 145784604816374866144131285430889962727208297722245411306711449302875041684
const G: [u64; 4] = [
    0x6070024f42f8ef94,
    0xad187148e11a6161,
    0x3f0464519c8b0fa5,
    0x005282db87529cfa,
];

// Number of bytes needed to represent field element
const ELEMENT_BYTES: usize = core::mem::size_of::<[u64; 4]>();

// FIELD ELEMENT
// ================================================================================================

/// Represents a base field element.
///
/// Internal values are stored in their canonical form in the range [0, M).
/// The backing type is stark_curve::BaseElementInner.
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
    /// Creates a new field element from a [u64; 4] value.
    /// The value is converted to Montgomery form by computing
    /// (a.R^0 * R^2) / R = a.R
    pub const fn new(value: [u64; 4]) -> Self {
        BaseElement(BaseElementInner::new(value))
    }

    #[inline]
    pub fn add(&self, rhs: &Self) -> Self {
        BaseElement(self.0.add(&rhs.0))
    }

    #[inline]
    pub fn sub(&self, rhs: &Self) -> Self {
        BaseElement(self.0.sub(&rhs.0))
    }

    #[inline]
    pub fn neg(&self) -> Self {
        BaseElement(self.0.neg())
    }

    #[inline]
    pub fn mul(&self, rhs: &Self) -> Self {
        BaseElement(self.0.mul(&rhs.0))
    }

    #[inline]
    pub fn square(&self) -> Self {
        BaseElement(self.0.square())
    }

    #[inline]
    pub fn double(&self) -> Self {
        self.add(self)
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let tmp = BaseElementInner::from_bytes(bytes);
        if bool::from(tmp.is_none()) {
            None
        } else {
            Some(BaseElement(tmp.unwrap()))
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Self {
        BaseElement(BaseElementInner::from_bytes_wide(bytes))
    }

    /// Returns whether or not this element is strictly lexicographically
    /// larger than its negation.
    pub fn lexicographically_largest(&self) -> bool {
        bool::from(self.0.lexicographically_largest())
    }

    /// Constructs an element of `BaseElement` without checking that it is
    /// canonical.
    pub const fn from_raw_unchecked(v: [u64; 4]) -> Self {
        BaseElement(BaseElementInner::from_raw_unchecked(v))
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
}

impl FieldElement for BaseElement {
    type Representation = Repr;

    type BaseField = Self;

    const ZERO: Self = BaseElement(BaseElementInner::zero());
    const ONE: Self = BaseElement(BaseElementInner::one());

    const ELEMENT_BYTES: usize = ELEMENT_BYTES;

    const IS_CANONICAL: bool = true;

    fn inv(self) -> Self {
        BaseElement(self.invert().unwrap_or(BaseElementInner::zero()))
    }

    fn conjugate(&self) -> Self {
        BaseElement(self.0)
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        // TODO: take endianness into account
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

        if (p as usize) % mem::align_of::<[u64; 4]>() != 0 {
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
        debug_assert_eq!(Self::ELEMENT_BYTES, mem::size_of::<[u64; 4]>());
        let result = vec![[0u64; 4]; n];

        // translate a zero-filled vector of [u64; 4]s into a vector of base field elements
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
    /// sage: MODULUS = 2^251 + 17 * 2^192 + 1 \
    /// sage: GF(MODULUS).is_prime_field() \
    /// True \
    /// sage: GF(MODULUS).order() \
    /// 3618502788666131213697322783095070105623107215331596699973092056135872020481
    const MODULUS: Self::Representation = Repr([
        0x0000000000000001,
        0x0000000000000000,
        0x0000000000000000,
        0x0800000000000011,
    ]);
    const MODULUS_BITS: u32 = 256;

    /// sage: GF(MODULUS).primitive_element() \
    /// 3
    const GENERATOR: Self = BaseElement::from_raw_unchecked([3, 0, 0, 0]);

    /// sage: is_odd((MODULUS - 1) / 2^192) \
    /// True
    const TWO_ADICITY: u32 = 192;

    /// sage: k = (MODULUS - 1) / 2^192 \
    /// sage: GF(MODULUS).primitive_element()^k \
    /// 145784604816374866144131285430889962727208297722245411306711449302875041684
    const TWO_ADIC_ROOT_OF_UNITY: Self = BaseElement::new(G);

    fn get_root_of_unity(n: u32) -> Self {
        BaseElement(BaseElementInner::get_root_of_unity(n))
    }

    fn get_modulus_le_bytes() -> Vec<u8> {
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0,
            0, 0, 8,
        ]
    }

    fn to_repr(&self) -> Self::Representation {
        Repr(self.0.output_reduced_limbs())
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

/// Quadratic extension for this field is not implemented as
/// it already provides a sufficient security level.
impl ExtensibleField<2> for BaseElement {
    fn mul(_a: [Self; 2], _b: [Self; 2]) -> [Self; 2] {
        unimplemented!()
    }

    fn frobenius(_x: [Self; 2]) -> [Self; 2] {
        unimplemented!()
    }

    fn is_supported() -> bool {
        false
    }
}

// CUBIC EXTENSION
// ================================================================================================

/// Cubic extension for this field is not implemented as
/// it already provides a sufficient security level.
impl ExtensibleField<3> for BaseElement {
    fn mul(_a: [Self; 3], _b: [Self; 3]) -> [Self; 3] {
        unimplemented!()
    }

    fn frobenius(_x: [Self; 3]) -> [Self; 3] {
        unimplemented!()
    }

    fn is_supported() -> bool {
        false
    }
}

// TYPE CONVERSIONS
// ================================================================================================

impl From<BaseElementInner> for BaseElement {
    /// Converts a 128-bit value into a field element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently preformed.
    fn from(value: BaseElementInner) -> Self {
        BaseElement(value)
    }
}

impl From<u128> for BaseElement {
    /// Converts a 128-bit value into a field element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently preformed.
    fn from(value: u128) -> Self {
        BaseElement(BaseElementInner::from(value))
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

impl From<[u8; 32]> for BaseElement {
    /// Converts the value encoded in an array of 32 bytes into a field element. The bytes
    /// are assumed to be in little-endian byte order. If the value is greater than or equal
    /// to the field modulus, modular reduction is silently preformed.
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_bytes(&bytes).unwrap_or(Self::ZERO)
    }
}

impl From<&[u8; 32]> for BaseElement {
    /// Converts the value encoded in an array of 32 bytes into a field element. The bytes
    /// are assumed to be in little-endian byte order. If the value is greater than or equal
    /// to the field modulus, modular reduction is silently preformed.
    fn from(bytes: &[u8; 32]) -> Self {
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

        let mut bytes: [u8; 32] = bytes[0..32].try_into().unwrap();
        // masking away the unused MSBs
        bytes[31] &= 0b0001_1111;

        match BaseElement::from_bytes(&bytes) {
            Some(e) => Ok(e),
            None => Err(DeserializationError::InvalidValue(
                "invalid field element: value is greater than or equal to the field modulus"
                    .to_string(),
            )),
        }
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
