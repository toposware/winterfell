use cheetah::Fp6;
use cheetah::{AffinePoint as AffinePointInner, ProjectivePoint as ProjectivePointInner};
use utils::{
    collections::Vec, string::ToString, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Serializable,
};

use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Deref, DerefMut, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::fields::f64::BaseElement;
use crate::StarkField;

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
        write!(f, "{self:?}")
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
    type Output = AffinePoint;

    fn mul(self, rhs: &'b Scalar) -> AffinePoint {
        AffinePoint(self.0.mul(rhs.0).into())
    }
}

impl<'a> Mul<Scalar> for &'a AffinePoint {
    type Output = AffinePoint;

    fn mul(self, rhs: Scalar) -> AffinePoint {
        AffinePoint(self.0.mul(rhs.0).into())
    }
}

impl Mul<Scalar> for AffinePoint {
    type Output = AffinePoint;

    fn mul(self, rhs: Scalar) -> AffinePoint {
        AffinePoint(self.0.mul(rhs.0).into())
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
        self.0.get_x().output_internal().map(BaseElement::new)
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> [BaseElement; 6] {
        self.0.get_y().output_internal().map(BaseElement::new)
    }

    pub fn generator() -> AffinePoint {
        AffinePoint(AffinePointInner::generator())
    }

    /// Constructs an `AffinePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 12]) -> Self {
        let x = Fp6::new([
            elems[0].to_repr(),
            elems[1].to_repr(),
            elems[2].to_repr(),
            elems[3].to_repr(),
            elems[4].to_repr(),
            elems[5].to_repr(),
        ]);
        let y = Fp6::new([
            elems[6].to_repr(),
            elems[7].to_repr(),
            elems[8].to_repr(),
            elems[9].to_repr(),
            elems[10].to_repr(),
            elems[11].to_repr(),
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
        write!(f, "{self:?}")
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
        self.0.get_x().output_internal().map(BaseElement::new)
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> [BaseElement; 6] {
        self.0.get_y().output_internal().map(BaseElement::new)
    }

    // Returns the z coordinate of this ProjectivePoint
    pub fn get_z(&self) -> [BaseElement; 6] {
        self.0.get_z().output_internal().map(BaseElement::new)
    }

    /// Returns a fixed generator of the group.
    pub fn generator() -> ProjectivePoint {
        ProjectivePoint(ProjectivePointInner::generator())
    }

    /// Constructs a `ProjectivePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 18]) -> Self {
        let x = Fp6::new([
            elems[0].to_repr(),
            elems[1].to_repr(),
            elems[2].to_repr(),
            elems[3].to_repr(),
            elems[4].to_repr(),
            elems[5].to_repr(),
        ]);
        let y = Fp6::new([
            elems[6].to_repr(),
            elems[7].to_repr(),
            elems[8].to_repr(),
            elems[9].to_repr(),
            elems[10].to_repr(),
            elems[11].to_repr(),
        ]);
        let z = Fp6::new([
            elems[12].to_repr(),
            elems[13].to_repr(),
            elems[14].to_repr(),
            elems[15].to_repr(),
            elems[16].to_repr(),
            elems[17].to_repr(),
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
        target.write_u8_slice(&self.0.to_compressed().to_bytes());
    }
}

impl Deserializable for AffinePoint {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        use cheetah::CompressedPoint;
        let bytes = source.read_u8_array()?;
        let pt = AffinePointInner::from_compressed(&CompressedPoint(bytes));
        if bool::from(pt.is_none()) {
            return Err(DeserializationError::InvalidValue(
                "Invalid point".to_string(),
            ));
        };
        Ok(AffinePoint(pt.unwrap()))
    }
}

impl Serializable for ProjectivePoint {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.0.to_compressed().to_bytes());
    }
}

impl Deserializable for ProjectivePoint {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        use cheetah::CompressedPoint;
        let bytes = source.read_u8_array()?;
        let pt = ProjectivePointInner::from_compressed(&CompressedPoint(bytes));
        if bool::from(pt.is_none()) {
            return Err(DeserializationError::InvalidValue(
                "Invalid point".to_string(),
            ));
        };
        Ok(ProjectivePoint(pt.unwrap()))
    }
}

// This module exports the unit tests from the underlying cheetah crate.
// The heavy coordinates handling is necessary for testing all the wrapped
// methods provided here, even though most of them won't be needed in this
// library and could be removed in a later iteration.
#[cfg(test)]
mod tests {
    use super::*;
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
        coordinates[0..6].copy_from_slice(&(gen.0.get_x() * z).output_internal());
        coordinates[6..12].copy_from_slice(&(gen.0.get_y() * z).output_internal());
        coordinates[12..18].copy_from_slice(&z.output_internal());
        let mut test =
            ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));

        assert!(bool::from(test.is_on_curve()));

        coordinates[0..6].copy_from_slice(&z.output_internal());
        test = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
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
        coordinates[0..6].copy_from_slice(&(a.0.get_x() * z).output_internal());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_internal());
        coordinates[12..18].copy_from_slice(&z.output_internal());
        let mut c = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
        assert!(bool::from(c.is_on_curve()));

        assert!(a == c);
        assert!(b != c);
        assert!(c == a);
        assert!(c != b);

        coordinates[6..12].copy_from_slice(&(-a.0.get_y() * z).output_internal());
        c = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
        assert!(bool::from(c.is_on_curve()));

        assert!(a != c);
        assert!(b != c);
        assert!(c != a);
        assert!(c != b);

        coordinates[0..6].copy_from_slice(&z.output_internal());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_internal());
        c = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
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
        coordinates[0..6].copy_from_slice(&(a.0.get_x() * z).output_internal());
        coordinates[6..12].copy_from_slice(&(a.0.get_y() * z).output_internal());
        coordinates[12..18].copy_from_slice(&z.output_internal());
        let c = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));

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
                    BaseElement::new(0x367b029f5f146e3d),
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
                coordinates[0..6].copy_from_slice(&(b.0.get_x() * z).output_internal());
                coordinates[6..12].copy_from_slice(&(b.0.get_y() * z).output_internal());
                coordinates[12..18].copy_from_slice(&z.output_internal());
                b = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
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
                coordinates[0..6].copy_from_slice(&(b.0.get_x() * z).output_internal());
                coordinates[6..12].copy_from_slice(&(b.0.get_y() * z).output_internal());
                coordinates[12..18].copy_from_slice(&z.output_internal());
                b = ProjectivePoint::from_raw_coordinates(coordinates.map(|e| BaseElement::new(e)));
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
            BaseElement::new(0x9bfcd3244afcb637),
            BaseElement::new(0x39005e478830b187),
            BaseElement::new(0x7046f1c03b42c6cc),
            BaseElement::new(0xb5eeac99193711e5),
            BaseElement::new(0x7fd272e724307b98),
            BaseElement::new(0xcc371dd6dd5d8625),
            BaseElement::new(0x9d03fdc216dfaae8),
            BaseElement::new(0xbf4ade2a7665d9b8),
            BaseElement::new(0xf08b022d5b3262b7),
            BaseElement::new(0x2eaf583a3cf15c6f),
            BaseElement::new(0xa92531e4b1338285),
            BaseElement::new(0x5b8157814141a7a7),
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
            BaseElement::new(0x9bfcd3244afcb637),
            BaseElement::new(0x39005e478830b187),
            BaseElement::new(0x7046f1c03b42c6cc),
            BaseElement::new(0xb5eeac99193711e5),
            BaseElement::new(0x7fd272e724307b98),
            BaseElement::new(0xcc371dd6dd5d8625),
            BaseElement::new(0x9d03fdc216dfaae8),
            BaseElement::new(0xbf4ade2a7665d9b8),
            BaseElement::new(0xf08b022d5b3262b7),
            BaseElement::new(0x2eaf583a3cf15c6f),
            BaseElement::new(0xa92531e4b1338285),
            BaseElement::new(0x5b8157814141a7a7),
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
}
