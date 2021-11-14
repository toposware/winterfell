use ::utils::{
    collections::Vec, string::ToString, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Serializable,
};
use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Deref, DerefMut, Mul, MulAssign, Neg, Sub, SubAssign};
use stark_curve::{AffinePoint as AffinePointInner, ProjectivePoint as ProjectivePointInner};

use crate::fields::f252::BaseElement;

mod scalar;
pub use scalar::Scalar;

// A = 1
// B = 3141592653589793238462643383279502884197169399375105820974944592307816406665
pub const B: BaseElement = BaseElement::from_raw_unchecked([
    0x359ddd67b59a21ca,
    0x6725f2237aab9006,
    0xab8a1e002a41f947,
    0x013931651774247f,
]);

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
    pub fn get_x(&self) -> BaseElement {
        self.0.get_x().into()
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> BaseElement {
        self.0.get_y().into()
    }

    // From StarkWare:
    // G = [
    //      874739451078007766457464989774322083649278607533249481151382481072868806602,
    //      152666792071518830868575557812948353041420400780739481342941381225525861407,
    // ]
    pub fn generator() -> AffinePoint {
        AffinePoint(AffinePointInner::generator())
    }

    pub fn to_compressed(&self) -> [u8; 32] {
        self.0.to_compressed()
    }

    pub fn to_uncompressed(&self) -> [u8; 64] {
        self.0.to_uncompressed()
    }

    /// Attempts to deserialize an uncompressed element.
    pub fn from_uncompressed(bytes: &[u8; 64]) -> Option<Self> {
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
    pub fn from_uncompressed_unchecked(bytes: &[u8; 64]) -> Option<Self> {
        let tmp = AffinePointInner::from_uncompressed_unchecked(bytes);
        if tmp.is_some().into() {
            Some(AffinePoint(tmp.unwrap()))
        } else {
            None
        }
    }

    /// Attempts to deserialize a compressed element.
    pub fn from_compressed(bytes: &[u8; 32]) -> Option<Self> {
        let tmp = AffinePointInner::from_compressed(bytes);
        if tmp.is_some().into() {
            Some(AffinePoint(tmp.unwrap()))
        } else {
            None
        }
    }

    /// Constructs an `AffinePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 2]) -> Self {
        AffinePoint(AffinePointInner::from_raw_coordinates([
            *elems[0], *elems[1],
        ]))
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

    pub fn multiply(&self, by: &[u8; 32]) -> AffinePoint {
        AffinePoint(self.0.multiply(by))
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
    pub fn get_x(&self) -> BaseElement {
        self.0.get_x().into()
    }

    // Returns the y coordinate of this ProjectivePoint
    pub fn get_y(&self) -> BaseElement {
        self.0.get_y().into()
    }

    // Returns the z coordinate of this ProjectivePoint
    pub fn get_z(&self) -> BaseElement {
        self.0.get_z().into()
    }

    /// Returns a fixed generator of the group.
    pub fn generator() -> ProjectivePoint {
        ProjectivePoint(ProjectivePointInner::generator())
    }

    /// Outputs a compress byte representation of this `ProjectivePoint` element
    pub fn to_compressed(&self) -> [u8; 32] {
        AffinePoint::from(self).to_compressed()
    }

    /// Outputs an uncompressed byte representation of this `ProjectivePoint` element
    /// It is twice larger than when calling `ProjectivePoint::to_uncompress()`
    pub fn to_uncompressed(&self) -> [u8; 64] {
        AffinePoint::from(self).to_uncompressed()
    }

    /// Attempts to deserialize an uncompressed element.
    pub fn from_uncompressed(bytes: &[u8; 64]) -> Option<Self> {
        AffinePoint::from_uncompressed(bytes).map(ProjectivePoint::from)
    }

    /// Attempts to deserialize an uncompressed element, not checking if the
    /// element is on the curve and not checking if it is in the correct subgroup.
    /// **This is dangerous to call unless you trust the bytes you are reading; otherwise,
    /// API invariants may be broken.** Please consider using `from_uncompressed()` instead.
    pub fn from_uncompressed_unchecked(bytes: &[u8; 64]) -> Option<Self> {
        AffinePoint::from_uncompressed_unchecked(bytes).map(ProjectivePoint::from)
    }

    /// Attempts to deserialize a compressed element.
    pub fn from_compressed(bytes: &[u8; 32]) -> Option<Self> {
        AffinePoint::from_compressed(bytes).map(ProjectivePoint::from)
    }

    /// Constructs a `ProjectivePoint` element without checking that it is a valid point.
    pub fn from_raw_coordinates(elems: [BaseElement; 3]) -> Self {
        ProjectivePoint(ProjectivePointInner::from_raw_coordinates([
            *elems[0], *elems[1], *elems[2],
        ]))
    }

    /// Computes the doubling of this point.
    pub fn double(&self) -> ProjectivePoint {
        ProjectivePoint(self.0.double())
    }

    /// Adds this point to another point.
    pub fn add(&self, rhs: &ProjectivePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add(&rhs.0))
    }

    /// Adds this point to another point in the affine model.
    pub fn add_mixed(&self, rhs: &AffinePoint) -> ProjectivePoint {
        ProjectivePoint(self.0.add_mixed(&rhs.0))
    }

    fn multiply(&self, by: &[u8; 32]) -> ProjectivePoint {
        ProjectivePoint(self.0.multiply(by))
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

        let z = BaseElement::from_raw_unchecked([
            0xba7a_fa1f_9a6f_e250,
            0xfa0f_5b59_5eaf_e731,
            0x64aa_6e06_49b2_078c,
            0x12b1_08ac_3364_3c3e,
        ]);

        let gen = AffinePoint::generator();
        let mut test = ProjectivePoint::from_raw_coordinates([gen.get_x() * z, gen.get_y() * z, z]);

        assert!(bool::from(test.is_on_curve()));

        test = ProjectivePoint::from_raw_coordinates([z, gen.get_y() * z, z]);
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

        let z = BaseElement::from_raw_unchecked([
            0xba7a_fa1f_9a6f_e250,
            0xfa0f_5b59_5eaf_e731,
            0x64aa_6e06_49b2_078c,
            0x12b1_08ac_3364_3c3e,
        ]);

        let mut c = ProjectivePoint::from_raw_coordinates([a.get_x() * z, a.get_y() * z, z]);
        assert!(bool::from(c.is_on_curve()));

        assert!(a == c);
        assert!(b != c);
        assert!(c == a);
        assert!(c != b);

        c = ProjectivePoint::from_raw_coordinates([a.get_x() * z, -(a.get_y() * z), z]);
        assert!(bool::from(c.is_on_curve()));

        assert!(a != c);
        assert!(b != c);
        assert!(c != a);
        assert!(c != b);

        c = ProjectivePoint::from_raw_coordinates([z, a.get_y() * z, z]);
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

        let z = BaseElement::from_raw_unchecked([
            0xba7a_fa1f_9a6f_e250,
            0xfa0f_5b59_5eaf_e731,
            0x64aa_6e06_49b2_078c,
            0x12b1_08ac_3364_3c3e,
        ]);

        let c = ProjectivePoint::from_raw_coordinates([a.get_x() * z, a.get_y() * z, z]);

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
                    BaseElement::from_raw_unchecked([
                        0xe615a450e1fdd9b5,
                        0xce619a1cc782d03f,
                        0x32f56eeb17ebf75b,
                        0x0436a838130c395e,
                    ]),
                    BaseElement::from_raw_unchecked([
                        0x41a737c065c63f91,
                        0xf021ba8dc8ac14cf,
                        0x65b817f8a6401d0a,
                        0x05761f5ec05b595f,
                    ])
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
                let z = BaseElement::from_raw_unchecked([
                    0xc17353b3d7a10e3d,
                    0xc2f5384ab42a5582,
                    0x0063b880ed197c70,
                    0x05aa8f6d202cf6ee,
                ]);

                b = ProjectivePoint::from_raw_coordinates([b.get_x() * z, b.get_y() * z, z]);
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == ProjectivePoint::generator());
        }
        {
            let a = ProjectivePoint::identity();
            let mut b = ProjectivePoint::generator();
            {
                let z = BaseElement::from_raw_unchecked([
                    0x95e25f0d6e182289,
                    0x73c356ed9f63259d,
                    0x2ea25dcdddb574ba,
                    0x062ff85a7fba2316,
                ]);

                b = ProjectivePoint::from_raw_coordinates([b.get_x() * z, b.get_y() * z, z]);
            }
            let c = b + a;
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

        // Degenerate case
        {
            let beta = BaseElement::from_raw_unchecked([
                0xbb8f98773bab1514,
                0xc045b4924a03b95d,
                0x50a948596949abc5,
                0x0501dc54d3237d00,
            ]);
            let beta = beta.square();
            let a = ProjectivePoint::generator().double();
            let b =
                ProjectivePoint::from_raw_coordinates([a.get_x() * beta, -a.get_y(), a.get_z()]);
            assert!(bool::from(a.is_on_curve()));
            assert!(bool::from(b.is_on_curve()));

            let c = a + b;
            assert_eq!(
                AffinePoint::from(c),
                AffinePoint::from_raw_coordinates([
                    BaseElement::from_raw_unchecked([
                        0xe77e9d05aae5fd36,
                        0x1c07b49204438a63,
                        0xc0aff314c28d1231,
                        0x063795f7b86d8530,
                    ]),
                    BaseElement::from_raw_unchecked([
                        0x44eb9d075ffd4dec,
                        0xcb1891e31e559732,
                        0xb07476f7de8c13ee,
                        0x053e0ee8cc479512,
                    ])
                ])
            );
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
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
                let z = BaseElement::from_raw_unchecked([
                    0x4bcd576928d4a7ea,
                    0x56b0d442f105f5a9,
                    0x933e54a5006e33c1,
                    0x0574e4134c4e753b,
                ]);

                b = ProjectivePoint::from_raw_coordinates([b.get_x() * z, b.get_y() * z, z]);
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == ProjectivePoint::generator());
        }
        {
            let a = AffinePoint::identity();
            let mut b = ProjectivePoint::generator();
            {
                let z = BaseElement::from_raw_unchecked([
                    0x3bdc_4776_94c3_06e7,
                    0x2149_be4b_3949_fa24,
                    0x64aa_6e06_49b2_078c,
                    0x12b1_08ac_3364_3c3e,
                ]);

                b = ProjectivePoint::from_raw_coordinates([b.get_x() * z, b.get_y() * z, z]);
            }
            let c = b + a;
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

        // Degenerate case
        {
            let beta = BaseElement::from_raw_unchecked([
                0xbb8f98773bab1514,
                0xc045b4924a03b95d,
                0x50a948596949abc5,
                0x0501dc54d3237d00,
            ]);
            let beta = beta.square();
            let a = ProjectivePoint::generator().double();
            let b =
                ProjectivePoint::from_raw_coordinates([a.get_x() * beta, -a.get_y(), a.get_z()]);
            let a = AffinePoint::from(a);
            assert!(bool::from(a.is_on_curve()));
            assert!(bool::from(b.is_on_curve()));

            let c = a + b;
            assert_eq!(
                AffinePoint::from(c),
                AffinePoint::from_raw_coordinates([
                    BaseElement::from_raw_unchecked([
                        0xe77e9d05aae5fd36,
                        0x1c07b49204438a63,
                        0xc0aff314c28d1231,
                        0x063795f7b86d8530,
                    ]),
                    BaseElement::from_raw_unchecked([
                        0x44eb9d075ffd4dec,
                        0xcb1891e31e559732,
                        0xb07476f7de8c13ee,
                        0x053e0ee8cc479512,
                    ])
                ])
            );
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
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
            0xef427d940c471145,
            0xf9d1c30637e9f84d,
            0x843a5b754596e86b,
            0x05b910f89b6b601c,
        ]);
        let b = Scalar::new([
            0xcdf47d5adc756906,
            0x381699324f082566,
            0x725be442943c3f0f,
            0x0701db10daaec421,
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
            0xb951ca4b11baeb8c,
            0xbd8bccd724d2d460,
            0x3520dbe0f992ab40,
            0x02a7506357d39b4e,
        ]);
        let b = Scalar::new([
            0x80996fb6c25f0316,
            0xa518a33400a43fdd,
            0x8e456b2de42d5671,
            0x0401b958b504dd68,
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
            let point = AffinePoint::from_raw_coordinates([BaseElement::ZERO, BaseElement::ZERO]);
            let bytes = point.to_compressed();
            let point_decompressed = AffinePoint::from_compressed(&bytes);
            assert!(point_decompressed.is_none());

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_compressed();
            let point_decompressed = ProjectivePoint::from_compressed(&bytes);
            assert!(point_decompressed.is_none());
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
            let point = AffinePoint::from_raw_coordinates([BaseElement::ZERO, BaseElement::ZERO]);
            let bytes = point.to_uncompressed();
            let point_decompressed = AffinePoint::from_uncompressed(&bytes);
            assert!(bool::from(point_decompressed.is_none()));

            let point = ProjectivePoint::from(&point);
            let bytes = point.to_uncompressed();
            let point_decompressed = ProjectivePoint::from_uncompressed(&bytes);
            assert!(bool::from(point_decompressed.is_none()));
        }
    }
}
