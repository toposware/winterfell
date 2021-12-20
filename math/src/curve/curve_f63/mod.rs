use ::utils::{
    collections::Vec, string::ToString, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Serializable,
};
use cheetah::Fp6;
use cheetah::{AffinePoint as AffinePointInner, ProjectivePoint as ProjectivePointInner};

use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Deref, DerefMut, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::fields::f63::BaseElement;

pub use cheetah::B;

mod scalar;
pub use scalar::Scalar;

#[derive(Copy, Clone, Debug)]
pub struct AffinePoint(pub(crate) AffinePointInner);

impl Deref for AffinePoint {
    type Target = AffinePointInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AffinePoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for AffinePoint {
    fn default() -> AffinePoint {
        AffinePoint::identity()
    }
}

impl fmt::Display for AffinePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> From<&'a ProjectivePoint> for AffinePoint {
    fn from(p: &'a ProjectivePoint) -> AffinePoint {
        AffinePoint(AffinePointInner::from(&p.0))
    }
}

impl From<ProjectivePoint> for AffinePoint {
    fn from(p: ProjectivePoint) -> AffinePoint {
        AffinePoint::from(&p)
    }
}

impl PartialEq for AffinePoint {
    fn eq(&self, other: &Self) -> bool {
        (self.is_identity() & other.is_identity())
            | ((!self.is_identity())
                & (!other.is_identity())
                & self.get_x().eq(&other.get_x())
                & self.get_y().eq(&other.get_y()))
    }
}

impl Eq for AffinePoint {}

impl<'a> Neg for &'a AffinePoint {
    type Output = AffinePoint;

    fn neg(self) -> AffinePoint {
        if self.is_identity() {
            AffinePoint::identity()
        } else {
            AffinePoint(self.0.neg())
        }
    }
}

impl Neg for AffinePoint {
    type Output = AffinePoint;

    fn neg(self) -> AffinePoint {
        -&self
    }
}

impl<'a, 'b> Add<&'b ProjectivePoint> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        rhs.add_mixed(self)
    }
}

impl<'a, 'b> Add<&'b AffinePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b AffinePoint) -> ProjectivePoint {
        self.add_mixed(rhs)
    }
}

impl<'b> Add<&'b ProjectivePoint> for AffinePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'a> Add<ProjectivePoint> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl Add<ProjectivePoint> for AffinePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'b> Add<&'b AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'a> Add<AffinePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl Add<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'a, 'b> Sub<&'b ProjectivePoint> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        self + (-rhs)
    }
}

impl<'a, 'b> Sub<&'b AffinePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b AffinePoint) -> ProjectivePoint {
        self + (-rhs)
    }
}

impl<'b> Sub<&'b ProjectivePoint> for AffinePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl<'a> Sub<ProjectivePoint> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl Sub<ProjectivePoint> for AffinePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl<'b> Sub<&'b AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl<'a> Sub<AffinePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl Sub<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl SubAssign<AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: AffinePoint) {
        *self = *self - rhs;
    }
}

impl AddAssign<AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: AffinePoint) {
        *self = *self + rhs;
    }
}

impl<'b> SubAssign<&'b AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &'b AffinePoint) {
        *self = *self - rhs;
    }
}

impl<'b> AddAssign<&'b AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &'b AffinePoint) {
        *self = *self + rhs;
    }
}

impl<'b> Mul<&'b Scalar> for AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: &'b Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl<'a> Mul<Scalar> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl Mul<Scalar> for AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl<T> Sum<T> for ProjectivePoint
where
    T: Borrow<ProjectivePoint>,
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        iter.fold(Self::identity(), |acc, item| acc + item.borrow())
    }
}

impl AffinePoint {
    pub fn identity() -> AffinePoint {
        AffinePoint(AffinePointInner::identity())
    }

    // Returns the x coordinate of this ProjectivePoint
    pub fn get_x(&self) -> [BaseElement; 6] {
        self.0.get_x().output_reduced_limbs().map(BaseElement::new)
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> [BaseElement; 6] {
        self.0.get_y().output_reduced_limbs().map(BaseElement::new)
    }

    pub fn generator() -> AffinePoint {
        AffinePoint(AffinePointInner::generator())
    }

    pub fn to_compressed(&self) -> [u8; 48] {
        self.0.to_compressed()
    }

    pub fn to_uncompressed(&self) -> [u8; 96] {
        self.0.to_uncompressed()
    }

    /// Attempts to deserialize an uncompressed element.
    pub fn from_uncompressed(bytes: &[u8; 96]) -> Option<Self> {
        Self::from_uncompressed_unchecked(bytes).and_then(|p| {
            if p.is_on_curve() {
                Some(p)
            } else {
                None
            }
        })
    }

    /// Attempts to deserialize an uncompressed element, not checking if the
    /// element is on the curve and not checking if it is in the correct subgroup.
    /// **This is dangerous to call unless you trust the bytes you are reading; otherwise,
    /// API invariants may be broken.** Please consider using `from_uncompressed()` instead.
    pub fn from_uncompressed_unchecked(bytes: &[u8; 96]) -> Option<Self> {
        let tmp = AffinePointInner::from_uncompressed_unchecked(bytes);
        if tmp.is_some().into() {
            Some(AffinePoint(tmp.unwrap()))
        } else {
            None
        }
    }

    /// Attempts to deserialize a compressed element.
    pub fn from_compressed(bytes: &[u8; 48]) -> Option<Self> {
        let tmp = AffinePointInner::from_compressed(bytes);
        if tmp.is_some().into() {
            Some(AffinePoint(tmp.unwrap()))
        } else {
            None
        }
    }

    /// Attempts to deserialize an uncompressed element, not checking if the
    /// element is in the correct subgroup.
    /// **This is dangerous to call unless you trust the bytes you are reading; otherwise,
    /// API invariants may be broken.** Please consider using `from_compressed()` instead.
    pub fn from_compressed_unchecked(bytes: &[u8; 48]) -> Option<Self> {
        let tmp = AffinePointInner::from_compressed_unchecked(bytes);
        if tmp.is_some().into() {
            Some(AffinePoint(tmp.unwrap()))
        } else {
            None
        }
    }

    /// Constructs an `AffinePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 12]) -> Self {
        let x = Fp6::from([
            elems[0].output_internal(),
            elems[1].output_internal(),
            elems[2].output_internal(),
            elems[3].output_internal(),
            elems[4].output_internal(),
            elems[5].output_internal(),
        ]);
        let y = Fp6::from([
            elems[6].output_internal(),
            elems[7].output_internal(),
            elems[8].output_internal(),
            elems[9].output_internal(),
            elems[10].output_internal(),
            elems[11].output_internal(),
        ]);
        AffinePoint(AffinePointInner::from_raw_coordinates([x, y]))
    }

    /// Returns true if this element is the identity (the point at infinity).
    pub fn is_identity(&self) -> bool {
        bool::from(self.0.is_identity())
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> bool {
        bool::from(self.0.is_on_curve())
    }

    #[must_use]
    pub fn multiply(&self, by: &[u8; 32]) -> AffinePoint {
        AffinePoint(self.0.multiply_vartime(by))
    }

    #[must_use]
    pub fn multiply_double(
        &self,
        rhs: &AffinePoint,
        by_lhs: &[u8; 32],
        by_rhs: &[u8; 32],
    ) -> AffinePoint {
        AffinePoint(self.0.multiply_double_vartime(&rhs.0, by_lhs, by_rhs))
    }

    /// Multiplies by the curve cofactor
    #[must_use]
    pub fn clear_cofactor(&self) -> AffinePoint {
        AffinePoint(self.0.clear_cofactor())
    }

    /// Returns true if this point is free of an $h$-torsion component.
    /// This should always return true unless an "unchecked" API was used.
    pub fn is_torsion_free(&self) -> bool {
        self.0.is_torsion_free().into()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ProjectivePoint(pub(crate) ProjectivePointInner);

impl Default for ProjectivePoint {
    fn default() -> ProjectivePoint {
        ProjectivePoint::identity()
    }
}

impl fmt::Display for ProjectivePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> From<&'a AffinePoint> for ProjectivePoint {
    fn from(p: &'a AffinePoint) -> ProjectivePoint {
        ProjectivePoint(ProjectivePointInner::from(&p.0))
    }
}

impl From<AffinePoint> for ProjectivePoint {
    fn from(p: AffinePoint) -> ProjectivePoint {
        ProjectivePoint::from(&p)
    }
}

impl PartialEq for ProjectivePoint {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for ProjectivePoint {}

impl<'a> Neg for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        ProjectivePoint(self.0.neg())
    }
}

impl Neg for ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        -&self
    }
}

impl<'a, 'b> Add<&'b ProjectivePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        self.add(rhs)
    }
}

impl<'b> Add<&'b ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'a> Add<ProjectivePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl Add<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(rhs.0))
    }
}

impl<'a, 'b> Sub<&'b ProjectivePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        self + (-rhs)
    }
}

impl<'b> Sub<&'b ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: &'b ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl<'a> Sub<ProjectivePoint> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl Sub<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, rhs: ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.sub(rhs.0))
    }
}

impl SubAssign<ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: ProjectivePoint) {
        *self = *self - rhs;
    }
}

impl AddAssign<ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: ProjectivePoint) {
        *self = *self + rhs;
    }
}

impl<'b> SubAssign<&'b ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, rhs: &'b ProjectivePoint) {
        *self = *self - rhs;
    }
}

impl<'b> AddAssign<&'b ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, rhs: &'b ProjectivePoint) {
        *self = *self + rhs;
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, other: &'b Scalar) -> Self::Output {
        self.multiply(&other.to_bytes())
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, other: &'b Scalar) -> Self::Output {
        ProjectivePoint::from(self).multiply(&other.to_bytes())
    }
}

impl<'b> Mul<&'b Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: &'b Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl<'a> Mul<Scalar> for &'a ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl Mul<Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, rhs: Scalar) -> ProjectivePoint {
        ProjectivePoint(self.0.mul(rhs.0))
    }
}

impl MulAssign<Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = *self * rhs;
    }
}

impl<'b> MulAssign<&'b Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, rhs: &'b Scalar) {
        *self = *self * rhs;
    }
}

impl Deref for ProjectivePoint {
    type Target = ProjectivePointInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProjectivePoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ProjectivePoint {
    /// Returns the identity of the group: the point at infinity.
    pub fn identity() -> ProjectivePoint {
        ProjectivePoint(ProjectivePointInner::identity())
    }

    // Returns the x coordinate of this ProjectivePoint
    pub fn get_x(&self) -> [BaseElement; 6] {
        self.0.get_x().output_reduced_limbs().map(BaseElement::new)
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> [BaseElement; 6] {
        self.0.get_y().output_reduced_limbs().map(BaseElement::new)
    }

    // Returns the z coordinate of this ProjectivePoint
    pub fn get_z(&self) -> [BaseElement; 6] {
        self.0.get_z().output_reduced_limbs().map(BaseElement::new)
    }

    /// Returns a fixed generator of the group.
    pub fn generator() -> ProjectivePoint {
        ProjectivePoint(ProjectivePointInner::generator())
    }

    /// Outputs a compress byte representation of this `ProjectivePoint` element
    pub fn to_compressed(&self) -> [u8; 48] {
        AffinePoint::from(self).to_compressed()
    }

    /// Outputs an uncompressed byte representation of this `ProjectivePoint` element
    /// It is twice larger than when calling `ProjectivePoint::to_uncompress()`
    pub fn to_uncompressed(&self) -> [u8; 96] {
        AffinePoint::from(self).to_uncompressed()
    }

    /// Attempts to deserialize an uncompressed element.
    pub fn from_uncompressed(bytes: &[u8; 96]) -> Option<Self> {
        AffinePoint::from_uncompressed(bytes).map(ProjectivePoint::from)
    }

    /// Attempts to deserialize an uncompressed element, not checking if the
    /// element is on the curve and not checking if it is in the correct subgroup.
    /// **This is dangerous to call unless you trust the bytes you are reading; otherwise,
    /// API invariants may be broken.** Please consider using `from_uncompressed()` instead.
    pub fn from_uncompressed_unchecked(bytes: &[u8; 96]) -> Option<Self> {
        AffinePoint::from_uncompressed_unchecked(bytes).map(ProjectivePoint::from)
    }

    /// Attempts to deserialize a compressed element.
    pub fn from_compressed(bytes: &[u8; 48]) -> Option<Self> {
        AffinePoint::from_compressed(bytes).map(ProjectivePoint::from)
    }

    /// Constructs a `ProjectivePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 18]) -> Self {
        let x = Fp6::from([
            elems[0].output_internal(),
            elems[1].output_internal(),
            elems[2].output_internal(),
            elems[3].output_internal(),
            elems[4].output_internal(),
            elems[5].output_internal(),
        ]);
        let y = Fp6::from([
            elems[6].output_internal(),
            elems[7].output_internal(),
            elems[8].output_internal(),
            elems[9].output_internal(),
            elems[10].output_internal(),
            elems[11].output_internal(),
        ]);
        let z = Fp6::from([
            elems[12].output_internal(),
            elems[13].output_internal(),
            elems[14].output_internal(),
            elems[15].output_internal(),
            elems[16].output_internal(),
            elems[17].output_internal(),
        ]);
        ProjectivePoint(ProjectivePointInner::from_raw_coordinates([x, y, z]))
    }

    /// Computes the doubling of this point.
    #[must_use]
    pub fn double(&self) -> ProjectivePoint {
        ProjectivePoint(self.0.double())
    }

    /// Adds this point to another point.
    #[must_use]
    pub fn add(&self, rhs: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(&rhs.0))
    }

    /// Adds this point to another point in the affine model.
    #[must_use]
    pub fn add_mixed(&self, rhs: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add_mixed(&rhs.0))
    }

    #[must_use]
    pub fn multiply(&self, by: &[u8; 32]) -> ProjectivePoint {
        ProjectivePoint(self.0.multiply_vartime(by))
    }

    #[must_use]
    pub fn multiply_double(
        &self,
        rhs: &ProjectivePoint,
        by_lhs: &[u8; 32],
        by_rhs: &[u8; 32],
    ) -> ProjectivePoint {
        ProjectivePoint(self.0.multiply_double_vartime(&rhs.0, by_lhs, by_rhs))
    }

    /// Multiplies by the curve cofactor
    #[must_use]
    pub fn clear_cofactor(&self) -> ProjectivePoint {
        ProjectivePoint(self.0.clear_cofactor())
    }

    /// Converts a batch of `G1Projective` elements into `AffinePoint` elements. This
    /// function will panic if `p.len() != q.len()`.
    pub fn batch_normalize(p: &[Self], q: &mut [AffinePoint]) {
        let p_inner: Vec<ProjectivePointInner> = p.iter().map(|e| e.0).collect();
        let mut res: Vec<AffinePointInner> = q.iter().map(|e| e.0).collect();

        ProjectivePointInner::batch_normalize(&p_inner, &mut res);
        for (index, elem) in q.iter_mut().enumerate() {
            *elem = AffinePoint(res[index]);
        }
    }

    /// Returns true if this element is the identity (the point at infinity).
    pub fn is_identity(&self) -> bool {
        bool::from(self.0.is_identity())
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> bool {
        bool::from(self.0.is_on_curve())
    }
}

// SERIALIZATION / DESERIALIZATION
// ------------------------------------------------------------------------------------------------

impl Serializable for AffinePoint {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.to_compressed());
    }
}

impl Deserializable for AffinePoint {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let bytes = source.read_u8_array()?;
        if bytes.len() < 32 {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                32,
                bytes.len(),
            )));
        }
        if bytes.len() > 32 {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                32,
                bytes.len(),
            )));
        }

        AffinePoint::from_compressed(&bytes)
            .ok_or_else(|| DeserializationError::UnknownError("".to_string()))
    }
}

impl Serializable for ProjectivePoint {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&AffinePoint::from(self).to_compressed());
    }
}

impl Deserializable for ProjectivePoint {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let bytes = source.read_u8_array()?;
        if bytes.len() < 32 {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                32,
                bytes.len(),
            )));
        }
        if bytes.len() > 32 {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                32,
                bytes.len(),
            )));
        }

        Ok(ProjectivePoint::from(
            AffinePoint::from_compressed(&bytes)
                .ok_or_else(|| DeserializationError::UnknownError("".to_string()))?,
        ))
    }
}

// This module exports the unit tests from the underlying cheetah crate.
// The heavy coordinates handling is necessary for testing all the wrapped
// methods provided here, even though most of them won't be needed in this
// library and could be removed in a later iteration.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::FieldElement;
    use rand_utils::rand_value;

    #[test]
    fn test_is_on_curve() {
        assert!(bool::from(AffinePoint::identity().is_on_curve()));
        assert!(bool::from(AffinePoint::generator().is_on_curve()));
        assert!(bool::from(ProjectivePoint::identity().is_on_curve()));
        assert!(bool::from(ProjectivePoint::generator().is_on_curve()));

        let z = Fp6::from_raw_unchecked([
            0x29eedd8f12973c87,
            0x341a681d86aa8bb4,
            0x3b0cf6ff269650b1,
            0x3361321304a4f391,
            0x152a4144440c5eb7,
            0x28f32bdf64c201d,
        ]);

        let gen = AffinePoint::generator();
        let mut coordinates = [0u64; 18];
        coordinates[0..6].copy_from_slice(&(gen.0.get_x() * z).output_unreduced_limbs());
        coordinates[6..12].copy_from_slice(&(gen.0.get_y() * z).output_unreduced_limbs());
        coordinates[12..18].copy_from_slice(&z.output_unreduced_limbs());
        let mut test = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );

        assert!(bool::from(test.is_on_curve()));

        coordinates[0..6].copy_from_slice(&z.output_unreduced_limbs());
        test = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );
        assert!(!bool::from(test.is_on_curve()));
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_affine_point_equality() {
        let a = AffinePoint::generator();
        let b = AffinePoint::identity();
        let c = AffinePoint::default();

        assert!(a == a);
        assert!(b == b);
        assert!(b == c);
        assert!(a != b);
        assert!(b != a);

        assert!(bool::from(b.is_identity()));
        assert!(!bool::from(a.eq(&b)));
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_projective_point_equality() {
        let a = ProjectivePoint::generator();
        let b = ProjectivePoint::identity();
        let c = ProjectivePoint::default();

        assert!(a == a);
        assert!(b == b);
        assert!(b == c);
        assert!(a != b);
        assert!(b != a);

        assert!(bool::from(b.is_identity()));
        assert!(!bool::from(a.eq(&b)));

        let z = Fp6::from_raw_unchecked([
            0x29eedd8f12973c87,
            0x341a681d86aa8bb4,
            0x3b0cf6ff269650b1,
            0x3361321304a4f391,
            0x152a4144440c5eb7,
            0x28f32bdf64c201d,
        ]);

        let mut coordinates = [0u64; 18];
        coordinates[0..6].copy_from_slice(&(a.0.get_x() * z).output_unreduced_limbs());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_unreduced_limbs());
        coordinates[12..18].copy_from_slice(&z.output_unreduced_limbs());
        let mut c = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );
        assert!(bool::from(c.is_on_curve()));

        assert!(a == c);
        assert!(b != c);
        assert!(c == a);
        assert!(c != b);

        coordinates[6..12].copy_from_slice(&(-a.0.get_y() * z).output_unreduced_limbs());
        c = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );
        assert!(bool::from(c.is_on_curve()));

        assert!(a != c);
        assert!(b != c);
        assert!(c != a);
        assert!(c != b);

        coordinates[0..6].copy_from_slice(&z.output_unreduced_limbs());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_unreduced_limbs());
        c = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );
        assert!(!bool::from(c.is_on_curve()));
        assert!(a != b);
        assert!(a != c);
        assert!(b != c);
    }

    #[test]
    fn test_projective_to_affine() {
        let a = ProjectivePoint::generator();
        let b = ProjectivePoint::identity();

        assert!(bool::from(AffinePoint::from(a).is_on_curve()));
        assert!(!bool::from(AffinePoint::from(a).is_identity()));
        assert!(bool::from(AffinePoint::from(b).is_on_curve()));
        assert!(bool::from(AffinePoint::from(b).is_identity()));

        let z = Fp6::from_raw_unchecked([
            0x29eedd8f12973c87,
            0x341a681d86aa8bb4,
            0x3b0cf6ff269650b1,
            0x3361321304a4f391,
            0x152a4144440c5eb7,
            0x28f32bdf64c201d,
        ]);

        let mut coordinates = [0u64; 18];
        coordinates[0..6].copy_from_slice(&(a.0.get_x() * z).output_unreduced_limbs());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_unreduced_limbs());
        coordinates[12..18].copy_from_slice(&z.output_unreduced_limbs());
        let c = ProjectivePoint::from_raw_coordinates(
            coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
        );

        assert_eq!(AffinePoint::from(c), AffinePoint::generator());
    }

    #[test]
    fn test_affine_to_projective() {
        let a = AffinePoint::generator();
        let b = AffinePoint::identity();

        assert!(bool::from(ProjectivePoint::from(a).is_on_curve()));
        assert!(!bool::from(ProjectivePoint::from(a).is_identity()));
        assert!(bool::from(ProjectivePoint::from(b).is_on_curve()));
        assert!(bool::from(ProjectivePoint::from(b).is_identity()));
    }

    #[test]
    fn test_doubling() {
        {
            let tmp = ProjectivePoint::identity().double();
            assert!(bool::from(tmp.is_identity()));
            assert!(bool::from(tmp.is_on_curve()));
        }
        {
            let tmp = ProjectivePoint::generator().double();
            assert!(!bool::from(tmp.is_identity()));
            assert!(bool::from(tmp.is_on_curve()));

            assert_eq!(
                AffinePoint::from(tmp),
                AffinePoint::from_raw_coordinates([
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
                ])
            );
        }
    }

    #[test]
    fn test_projective_addition() {
        {
            let a = ProjectivePoint::identity();
            let b = ProjectivePoint::identity();
            let c = a + b;
            assert!(bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
        {
            let a = ProjectivePoint::identity();
            let mut b = ProjectivePoint::generator();
            {
                let z = Fp6::from_raw_unchecked([
                    0x29eedd8f12973c87,
                    0x341a681d86aa8bb4,
                    0x3b0cf6ff269650b1,
                    0x3361321304a4f391,
                    0x152a4144440c5eb7,
                    0x28f32bdf64c201d,
                ]);

                let mut coordinates = [0u64; 18];
                coordinates[0..6].copy_from_slice(&(b.0.get_x() * z).output_unreduced_limbs());
                coordinates[6..12].copy_from_slice(&(b.0.get_y() * z).output_unreduced_limbs());
                coordinates[12..18].copy_from_slice(&z.output_unreduced_limbs());
                b = ProjectivePoint::from_raw_coordinates(
                    coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
                );
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == ProjectivePoint::generator());
        }
        {
            let a = ProjectivePoint::generator().double().double(); // 4P
            let b = ProjectivePoint::generator().double(); // 2P
            let c = a + b;

            let mut d = ProjectivePoint::generator();
            for _ in 0..5 {
                d += ProjectivePoint::generator();
            }
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(!bool::from(d.is_identity()));
            assert!(bool::from(d.is_on_curve()));
            assert_eq!(c, d);
        }
    }

    #[test]
    fn test_mixed_addition() {
        {
            let a = AffinePoint::identity();
            let b = ProjectivePoint::identity();
            let c = a + b;
            assert!(bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
        {
            let a = AffinePoint::identity();
            let mut b = ProjectivePoint::generator();
            {
                let z = Fp6::from_raw_unchecked([
                    0x29eedd8f12973c87,
                    0x341a681d86aa8bb4,
                    0x3b0cf6ff269650b1,
                    0x3361321304a4f391,
                    0x152a4144440c5eb7,
                    0x28f32bdf64c201d,
                ]);

                let mut coordinates = [0u64; 18];
                coordinates[0..6].copy_from_slice(&(b.0.get_x() * z).output_unreduced_limbs());
                coordinates[6..12].copy_from_slice(&(b.0.get_y() * z).output_unreduced_limbs());
                coordinates[12..18].copy_from_slice(&z.output_unreduced_limbs());
                b = ProjectivePoint::from_raw_coordinates(
                    coordinates.map(|e| BaseElement::from_raw_unchecked(e)),
                );
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == ProjectivePoint::generator());
        }
        {
            let a = ProjectivePoint::generator().double().double(); // 4P
            let b = ProjectivePoint::generator().double(); // 2P
            let c = a + b;

            let mut d = ProjectivePoint::generator();
            for _ in 0..5 {
                d += AffinePoint::generator();
            }
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(!bool::from(d.is_identity()));
            assert!(bool::from(d.is_on_curve()));
            assert_eq!(c, d);
        }
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_projective_negation_and_subtraction() {
        let a = ProjectivePoint::generator().double();
        assert_eq!(a + (-a), ProjectivePoint::identity());
        assert_eq!(a + (-a), a - a);
    }

    #[test]
    fn test_affine_negation_and_subtraction() {
        let a = AffinePoint::generator();
        assert_eq!(ProjectivePoint::from(a) + (-a), ProjectivePoint::identity());
        assert_eq!(
            ProjectivePoint::from(a) + (-a),
            ProjectivePoint::from(a) - a
        );
    }

    #[test]
    fn test_projective_scalar_multiplication() {
        let g = ProjectivePoint::generator();
        let a = Scalar::new([
            0x1fe3ac3d0fde1429,
            0xd1ab3020993395ec,
            0x7b05ba9afe7bb36a,
            0x1a52ef1d2291d9bc,
        ]);
        let b = Scalar::new([
            0xb2a7f9f8569e3b44,
            0x1f9ada6e71c9167b,
            0xb73915944013806b,
            0x090e3287fea5247a,
        ]);
        let c = a * b;

        assert_eq!((g * a) * b, g * c);

        for _ in 0..100 {
            let a: Scalar = rand_value();
            let b: Scalar = rand_value();
            let c = a * b;

            assert_eq!((g * a) * b, g * c);
        }
    }

    #[test]
    fn test_affine_scalar_multiplication() {
        let g = AffinePoint::generator();
        let a = Scalar::new([
            0x1fe3ac3d0fde1429,
            0xd1ab3020993395ec,
            0x7b05ba9afe7bb36a,
            0x1a52ef1d2291d9bc,
        ]);
        let b = Scalar::new([
            0xb2a7f9f8569e3b44,
            0x1f9ada6e71c9167b,
            0xb73915944013806b,
            0x090e3287fea5247a,
        ]);
        let c = a * b;

        assert_eq!(AffinePoint::from(g * a) * b, g * c);

        for _ in 0..100 {
            let a: Scalar = rand_value();
            let b: Scalar = rand_value();
            let c = a * b;

            assert_eq!((g * a) * b, g * c);
        }
    }

    #[test]
    fn test_clear_cofactor() {
        // the generator (and the identity) are always on the curve
        let generator = ProjectivePoint::generator();
        assert!(bool::from(generator.clear_cofactor().is_on_curve()));
        let id = ProjectivePoint::identity();
        assert!(bool::from(id.clear_cofactor().is_on_curve()));

        let point = ProjectivePoint::from(&AffinePoint::from_raw_coordinates([
            BaseElement::from_raw_unchecked(0x30b857b59c073adf),
            BaseElement::from_raw_unchecked(0x32f03638832472c1),
            BaseElement::from_raw_unchecked(0x13e9b9fb403eeb05),
            BaseElement::from_raw_unchecked(0x372a0e4597af835f),
            BaseElement::from_raw_unchecked(0x24ea2fa836890130),
            BaseElement::from_raw_unchecked(0x35efbbad95df1753),
            BaseElement::from_raw_unchecked(0x15af8776c2b621ea),
            BaseElement::from_raw_unchecked(0x33c482433d49e4af),
            BaseElement::from_raw_unchecked(0x169525890222c375),
            BaseElement::from_raw_unchecked(0x22b58bc677671fe),
            BaseElement::from_raw_unchecked(0x32362b2e277aafea),
            BaseElement::from_raw_unchecked(0x1b7114359345ab3),
        ]));

        assert!(point.is_on_curve());
        assert!(!AffinePoint::from(point).is_torsion_free());
        let cleared_point = point.clear_cofactor();
        assert!(bool::from(cleared_point.is_on_curve()));
        assert!(AffinePoint::from(cleared_point).is_torsion_free());
    }

    #[test]
    fn test_is_torsion_free() {
        let a = AffinePoint::from_raw_coordinates([
            BaseElement::from_raw_unchecked(0x30b857b59c073adf),
            BaseElement::from_raw_unchecked(0x32f03638832472c1),
            BaseElement::from_raw_unchecked(0x13e9b9fb403eeb05),
            BaseElement::from_raw_unchecked(0x372a0e4597af835f),
            BaseElement::from_raw_unchecked(0x24ea2fa836890130),
            BaseElement::from_raw_unchecked(0x35efbbad95df1753),
            BaseElement::from_raw_unchecked(0x15af8776c2b621ea),
            BaseElement::from_raw_unchecked(0x33c482433d49e4af),
            BaseElement::from_raw_unchecked(0x169525890222c375),
            BaseElement::from_raw_unchecked(0x22b58bc677671fe),
            BaseElement::from_raw_unchecked(0x32362b2e277aafea),
            BaseElement::from_raw_unchecked(0x1b7114359345ab3),
        ]);

        assert!(bool::from(a.is_on_curve()));
        assert!(!bool::from(a.is_torsion_free()));
        assert!(bool::from(AffinePoint::identity().is_torsion_free()));
        assert!(bool::from(AffinePoint::generator().is_torsion_free()));
    }

    #[test]
    fn test_batch_normalize() {
        let a = ProjectivePoint::generator().double();
        let b = a.double();
        let c = b.double();

        for a_identity in (0..1).map(|n| n == 1) {
            for b_identity in (0..1).map(|n| n == 1) {
                for c_identity in (0..1).map(|n| n == 1) {
                    let mut v = [a, b, c];
                    if a_identity {
                        v[0] = ProjectivePoint::identity()
                    }
                    if b_identity {
                        v[1] = ProjectivePoint::identity()
                    }
                    if c_identity {
                        v[2] = ProjectivePoint::identity()
                    }

                    let mut t = [
                        AffinePoint::identity(),
                        AffinePoint::identity(),
                        AffinePoint::identity(),
                    ];
                    let expected = [
                        AffinePoint::from(v[0]),
                        AffinePoint::from(v[1]),
                        AffinePoint::from(v[2]),
                    ];

                    ProjectivePoint::batch_normalize(&v[..], &mut t[..]);

                    assert_eq!(&t[..], &expected[..]);
                }
            }
        }
    }

    // POINT COMPRESSION
    // ================================================================================================

    #[test]
    fn test_point_compressed() {
        // Random points
        for _ in 0..100 {
            let point = AffinePoint::from(AffinePoint::generator() * rand_value::<Scalar>());
            let bytes = point.to_compressed();
            let point_decompressed = AffinePoint::from_compressed(&bytes).unwrap();
            assert_eq!(point, point_decompressed);

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_compressed();
            let point_decompressed = ProjectivePoint::from_compressed(&bytes).unwrap();
            assert_eq!(point, point_decompressed);
        }

        // Identity point
        {
            let bytes = AffinePoint::identity().to_compressed();
            let point_decompressed = AffinePoint::from_compressed(&bytes).unwrap();
            assert!(bool::from(point_decompressed.is_identity()));

            let bytes = ProjectivePoint::identity().to_compressed();
            let point_decompressed = ProjectivePoint::from_compressed(&bytes).unwrap();
            assert!(bool::from(point_decompressed.is_identity()));
        }

        // Invalid points
        {
            let point = AffinePoint::from_raw_coordinates([BaseElement::ZERO; 12]);
            let bytes = point.to_compressed();
            let point_decompressed = AffinePoint::from_compressed(&bytes);
            assert!(point_decompressed.is_none());

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_compressed();
            let point_decompressed = ProjectivePoint::from_compressed(&bytes);
            assert!(point_decompressed.is_none());
        }
        {
            let bytes = [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ];
            let point_decompressed = AffinePoint::from_compressed_unchecked(&bytes);
            assert!(bool::from(point_decompressed.is_none()));
        }
    }

    #[test]
    fn test_point_uncompressed() {
        // Random points
        for _ in 0..100 {
            let point = AffinePoint::from(AffinePoint::generator() * rand_value::<Scalar>());
            let bytes = point.to_uncompressed();
            let point_decompressed = AffinePoint::from_uncompressed(&bytes).unwrap();
            assert_eq!(point, point_decompressed);

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_uncompressed();
            let point_decompressed = ProjectivePoint::from_uncompressed(&bytes).unwrap();
            assert_eq!(point, point_decompressed);
        }

        // Identity point
        {
            let bytes = AffinePoint::identity().to_uncompressed();
            let point_decompressed = AffinePoint::from_uncompressed(&bytes).unwrap();
            assert!(bool::from(point_decompressed.is_identity()));

            let bytes = ProjectivePoint::identity().to_uncompressed();
            let point_decompressed = ProjectivePoint::from_uncompressed(&bytes).unwrap();
            assert!(bool::from(point_decompressed.is_identity()));
        }

        // Invalid points
        {
            let point = AffinePoint::from_raw_coordinates([BaseElement::ZERO; 12]);
            let bytes = point.to_uncompressed();
            let point_decompressed = AffinePoint::from_uncompressed(&bytes);
            assert!(bool::from(point_decompressed.is_none()));

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_uncompressed();
            let point_decompressed = ProjectivePoint::from_uncompressed(&bytes);
            assert!(bool::from(point_decompressed.is_none()));
        }
        {
            let bytes = [
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let point_decompressed = AffinePoint::from_uncompressed_unchecked(&bytes);
            assert!(bool::from(point_decompressed.is_none()));

            let point_decompressed = ProjectivePoint::from_uncompressed_unchecked(&bytes);
            assert!(bool::from(point_decompressed.is_none()));
        }
        {
            let bytes = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            ];
            let point_decompressed = AffinePoint::from_uncompressed_unchecked(&bytes);
            assert!(bool::from(point_decompressed.is_none()));

            let point_decompressed = ProjectivePoint::from_uncompressed_unchecked(&bytes);
            assert!(bool::from(point_decompressed.is_none()));
        }
    }
}
