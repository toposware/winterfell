use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use bitvec::{order::Lsb0, slice::BitSlice};
use stark_curve::Scalar as ScalarInner;
use utils::{
    string::ToString, ByteReader, ByteWriter, Deserializable, DeserializationError, Randomizable,
    Serializable,
};

// CONSTANTS
// ================================================================================================

// Number of bytes needed to represent a scalar element
const ELEMENT_BYTES: usize = core::mem::size_of::<[u64; 4]>();

// SCALAR FIELD ELEMENT
// ================================================================================================

/// Represents a scalar field element.
///
/// Internal values are stored in their canonical form in the range [0, M).
/// The backing type is `stark_curve::Scalar`.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Scalar(pub(crate) ScalarInner);

impl Deref for Scalar {
    type Target = ScalarInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scalar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for Scalar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tmp = self.to_bytes();
        write!(f, "0x")?;
        for &b in tmp.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl Display for Scalar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Randomizable for Scalar {
    const VALUE_SIZE: usize = ELEMENT_BYTES;

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(bytes).ok()
    }
}

impl Scalar {
    pub fn zero() -> Self {
        Scalar(ScalarInner::zero())
    }

    pub fn one() -> Self {
        Scalar(ScalarInner::one())
    }

    /// Creates a new field element from a [u64; 4] value.
    /// The value is converted to Montgomery form by computing
    /// (a.R^0 * R^2) / R = a.R
    pub const fn new(value: [u64; 4]) -> Self {
        Scalar(ScalarInner::new(value))
    }

    pub fn add(&self, rhs: &Self) -> Self {
        Scalar(self.0.add(&rhs.0))
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        Scalar(self.0.sub(&rhs.0))
    }

    pub fn neg(&self) -> Self {
        Scalar(self.0.neg())
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Scalar(self.0.mul(&rhs.0))
    }

    pub fn square(&self) -> Self {
        Scalar(self.0.square())
    }

    pub fn double(&self) -> Self {
        self.add(self)
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let tmp = ScalarInner::from_bytes(bytes);
        if bool::from(tmp.is_none()) {
            None
        } else {
            Some(Scalar(tmp.unwrap()))
        }
    }

    /// Convert a little-endian bit sequence into a Scalar element
    pub fn from_bits(bit_slice: &BitSlice<Lsb0, u8>) -> Scalar {
        Scalar(ScalarInner::from_bits(bit_slice))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Returns whether or not this element is strictly lexicographically
    /// larger than its negation.
    pub fn lexicographically_largest(&self) -> bool {
        bool::from(self.0.lexicographically_largest())
    }

    pub fn exp(self, by: &[u64; 4]) -> Self {
        Scalar(self.0.exp(by))
    }

    pub fn invert(self) -> Self {
        Scalar(self.0.invert().unwrap_or_else(ScalarInner::zero))
    }

    pub fn conjugate(&self) -> Self {
        Scalar(self.0)
    }

    #[cfg(test)]
    pub const fn from_raw_unchecked(v: [u64; 4]) -> Self {
        Scalar(stark_curve::Scalar::from_raw_unchecked(v))
    }
}

// OVERLOADED OPERATORS
// ================================================================================================

impl<'a> Neg for &'a Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        self.neg()
    }
}

impl Neg for Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        -&self
    }
}

impl<'a, 'b> Sub<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn sub(self, rhs: &'b Scalar) -> Scalar {
        self.sub(rhs)
    }
}

impl<'a, 'b> Add<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn add(self, rhs: &'b Scalar) -> Scalar {
        self.add(rhs)
    }
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, rhs: Scalar) -> Scalar {
        Scalar(self.0 - rhs.0)
    }
}

impl Add<Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, rhs: Scalar) -> Scalar {
        Scalar(self.0 + rhs.0)
    }
}

impl SubAssign<Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: Scalar) {
        *self = *self - rhs;
    }
}

impl AddAssign<Scalar> for Scalar {
    fn add_assign(&mut self, rhs: Scalar) {
        *self = *self + rhs;
    }
}

impl<'b> SubAssign<&'b Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: &'b Scalar) {
        *self = &*self - rhs;
    }
}

impl<'b> AddAssign<&'b Scalar> for Scalar {
    fn add_assign(&mut self, rhs: &'b Scalar) {
        *self = &*self + rhs;
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, rhs: Scalar) -> Scalar {
        Scalar(self.0 * rhs.0)
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn mul(self, rhs: &'b Scalar) -> Scalar {
        self.mul(rhs)
    }
}

impl MulAssign<Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = *self * rhs;
    }
}

impl<'b> MulAssign<&'b Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: &'b Scalar) {
        *self = &*self * rhs;
    }
}

impl Div for Scalar {
    type Output = Self;

    fn div(self, rhs: Self) -> Scalar {
        self.mul(rhs.invert())
    }
}

impl DivAssign for Scalar {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

// TYPE CONVERSIONS
// ================================================================================================

impl From<ScalarInner> for Scalar {
    /// Converts a 128-bit value into a field element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently preformed.
    fn from(value: ScalarInner) -> Self {
        Scalar(value)
    }
}

impl From<u128> for Scalar {
    /// Converts a 128-bit value into a field element. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently preformed.
    fn from(value: u128) -> Self {
        let value_high: u64 = (value >> 64).try_into().unwrap();
        let value_low: u64 = (value & (u64::MAX as u128)).try_into().unwrap();
        Scalar::new([value_low, value_high, 0, 0])
    }
}

impl From<u64> for Scalar {
    /// Converts a 64-bit value into a field element.
    fn from(value: u64) -> Self {
        Scalar(ScalarInner::from(value))
    }
}

impl From<u32> for Scalar {
    /// Converts a 32-bit value into a field element.
    fn from(value: u32) -> Self {
        Scalar(ScalarInner::from(value))
    }
}

impl From<u16> for Scalar {
    /// Converts a 16-bit value into a field element.
    fn from(value: u16) -> Self {
        Scalar(ScalarInner::from(value))
    }
}

impl From<u8> for Scalar {
    /// Converts an 8-bit value into a field element.
    fn from(value: u8) -> Self {
        Scalar(ScalarInner::from(value))
    }
}

impl From<[u8; 32]> for Scalar {
    /// Converts the value encoded in an array of 32 bytes into a field element. The bytes
    /// are assumed to be in little-endian byte order. If the value is greater than or equal
    /// to the field modulus, modular reduction is silently preformed.
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_bytes(&bytes).unwrap_or_else(Self::zero)
    }
}

impl<'a> TryFrom<&'a [u8]> for Scalar {
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

        match Scalar::from_bytes(&bytes) {
            Some(e) => Ok(e),
            None => Err(DeserializationError::InvalidValue(
                "invalid field element: value is greater than or equal to the field modulus"
                    .to_string(),
            )),
        }
    }
}

// SERIALIZATION / DESERIALIZATION
// ------------------------------------------------------------------------------------------------

impl Serializable for Scalar {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.to_bytes());
    }
}

impl Deserializable for Scalar {
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

        Ok(Scalar::from_bytes(&bytes).unwrap_or_else(Self::zero))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::view::AsBits;
    use rand_utils::rand_value;

    const LARGEST: Scalar = Scalar::from_raw_unchecked([
        0x1e66a241adc64d2e,
        0xb781126dcae7b232,
        0xffffffffffffffff,
        0x0800000000000010,
    ]);

    // DISPLAY
    // ================================================================================================

    #[test]
    fn test_debug() {
        assert_eq!(
            format!("{:?}", Scalar::zero()),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            format!("{:?}", Scalar::one()),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
    }

    #[test]
    fn test_output_reduced_limbs() {
        assert_eq!(
            format!("{:?}", Scalar::zero().output_reduced_limbs()),
            "[0, 0, 0, 0]"
        );
        assert_eq!(
            format!("{:?}", Scalar::one().output_reduced_limbs()),
            "[1, 0, 0, 0]"
        );
    }

    // BASIC ALGEBRA
    // ================================================================================================

    #[test]
    fn test_equality() {
        assert_eq!(Scalar::default(), Scalar::zero());
        assert_eq!(Scalar::zero(), Scalar::zero());
        assert_eq!(Scalar::one(), Scalar::one());

        assert!(bool::from(Scalar::default().is_zero()));
        assert!(!bool::from(Scalar::zero().eq(&Scalar::one())));

        assert!(Scalar::zero() != Scalar::one());
    }

    #[test]
    fn test_addition() {
        let mut tmp = LARGEST;
        tmp += &LARGEST;

        assert_eq!(
            tmp,
            Scalar::from_raw_unchecked([
                0x1e66a241adc64d2d,
                0xb781126dcae7b232,
                0xffffffffffffffff,
                0x0800000000000010,
            ])
        );

        assert_eq!(tmp, LARGEST.double());

        let mut tmp = LARGEST;
        tmp += &Scalar::from_raw_unchecked([1, 0, 0, 0]);

        assert_eq!(tmp, Scalar::zero());
    }

    #[test]
    fn test_subtraction() {
        let mut tmp = LARGEST;
        tmp -= &LARGEST;

        assert_eq!(tmp, Scalar::zero());

        assert_eq!(Scalar::one() - Scalar::zero(), Scalar::one());
    }

    #[test]
    fn test_negation() {
        let tmp = -&LARGEST;

        assert_eq!(tmp, Scalar::from_raw_unchecked([1, 0, 0, 0]));

        let tmp = -&Scalar::zero();
        assert_eq!(tmp, Scalar::zero());
        let tmp = -&Scalar::from_raw_unchecked([1, 0, 0, 0]);
        assert_eq!(tmp, LARGEST);
    }

    #[test]
    fn test_multiplication() {
        let mut cur = LARGEST;

        for _ in 0..100 {
            let mut tmp = cur;
            tmp *= &cur;

            let mut tmp2 = Scalar::zero();
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
        assert_eq!(Scalar::zero().invert(), Scalar::zero());
        assert_eq!(Scalar::one().invert(), Scalar::one());
        assert_eq!((-&Scalar::one()).invert(), -&Scalar::one());

        let mut tmp: Scalar = rand_value();

        for _ in 0..100 {
            let mut tmp2 = tmp.invert();
            tmp2.mul_assign(&tmp);

            assert_eq!(tmp2, Scalar::one());

            tmp.add_assign(&rand_value());
        }
    }

    #[test]
    fn test_squaring() {
        let mut cur = LARGEST;

        for _ in 0..100 {
            let mut tmp = cur;
            let pow2 = tmp.exp(&[2, 0, 0, 0]);
            tmp = tmp.square();

            let mut tmp2 = Scalar::zero();
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
        let q_minus_2 = [
            0x1e66a241adc64d2d,
            0xb781126dcae7b232,
            0xffffffffffffffff,
            0x0800000000000010,
        ];

        let mut r1: Scalar = rand_value();
        let mut r2 = r1;

        for _ in 0..100 {
            r1 = r1.invert();
            r2 = r2.exp(&q_minus_2);

            assert_eq!(r1, r2);
            // Add r2 so we check something different next time around
            r1.add_assign(&r2);
            r2 = r1;
        }
    }

    #[test]
    fn test_conjugate() {
        let a: Scalar = rand_value();
        let b = a.conjugate();
        assert_eq!(a, b);
    }

    // SERIALIZATION / DESERIALIZATION
    // ================================================================================================

    #[test]
    fn test_to_bytes() {
        assert_eq!(
            Scalar::zero().to_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        assert_eq!(
            Scalar::one().to_bytes(),
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        assert_eq!(
            (-&Scalar::one()).to_bytes(),
            [
                46, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
                255, 255, 255, 255, 255, 255, 16, 0, 0, 0, 0, 0, 0, 8,
            ]
        );
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            Scalar::from_bytes(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
            .unwrap(),
            Scalar::zero()
        );

        assert_eq!(
            Scalar::from_bytes(&[
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
            .unwrap(),
            Scalar::one()
        );

        // -1 should work
        assert_eq!(
            Scalar::from_bytes(&[
                46, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
                255, 255, 255, 255, 255, 255, 16, 0, 0, 0, 0, 0, 0, 8,
            ])
            .unwrap(),
            -Scalar::one(),
        );

        // M is invalid
        assert!(bool::from(
            Scalar::from_bytes(&[
                47, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
                255, 255, 255, 255, 255, 255, 16, 0, 0, 0, 0, 0, 0, 8,
            ])
            .is_none()
        ));

        // Anything larger than the M is invalid
        assert!(bool::from(
            Scalar::from_bytes(&[
                48, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
                255, 255, 255, 255, 255, 255, 16, 0, 0, 0, 0, 0, 0, 8,
            ])
            .is_none()
        ));
        assert!(bool::from(
            Scalar::from_bytes(&[
                47, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
                255, 255, 255, 255, 255, 255, 16, 1, 0, 0, 0, 0, 0, 8,
            ])
            .is_none()
        ));
        assert!(bool::from(
            Scalar::from_bytes(&[
                153, 138, 183, 98, 118, 85, 192, 138, 212, 50, 253, 172, 212, 143, 5, 70, 43, 226,
                210, 217, 197, 56, 216, 63, 17, 0, 0, 0, 0, 0, 0, 12,
            ])
            .is_none()
        ));
    }

    #[test]
    fn test_from_bits() {
        let bytes = Scalar::zero().to_bytes();
        assert_eq!(Scalar::from_bits(bytes.as_bits::<Lsb0>()), Scalar::zero());

        let bytes = Scalar::one().to_bytes();
        assert_eq!(Scalar::from_bits(bytes.as_bits::<Lsb0>()), Scalar::one());

        // -1 should work
        let bytes = (-Scalar::one()).to_bytes();
        assert_eq!(Scalar::from_bits(bytes.as_bits::<Lsb0>()), -Scalar::one());

        // Modulus results in Scalar::zero()
        let bytes = [
            47, 77, 198, 173, 65, 162, 102, 30, 50, 178, 231, 202, 109, 18, 129, 183, 255, 255,
            255, 255, 255, 255, 255, 255, 16, 0, 0, 0, 0, 0, 0, 8,
        ];
        assert_eq!(Scalar::from_bits(bytes.as_bits::<Lsb0>()), Scalar::zero());
    }

    #[test]
    fn test_lexicographically_largest() {
        // a = 1664996970783755818040997309868806588776880551131535946003276210761949015868
        let a = Scalar::from_raw_unchecked([
            0xb754ad5b0d886f3c,
            0x972eb2efe92051c7,
            0x6509bb3aa3fbe7a2,
            0x03ae5ada0c8fdf58,
        ]);

        // b = 1953505817882375395656325473226263516749863200584551543150803247122563849715
        let b = Scalar::from_raw_unchecked([
            0x6711f4e6a03dddf3,
            0x20525f7de1c7606a,
            0x9af644c55c04185d,
            0x0451a525f37020b8,
        ]);

        assert_eq!(a.square(), b.square());
        assert!(!bool::from(a.lexicographically_largest()));
        assert!(bool::from(b.lexicographically_largest()));
    }

    // INITIALIZATION
    // ================================================================================================

    #[test]
    fn test_from_int() {
        let n = 42u8;
        let element = Scalar::from(n);

        assert_eq!(element, Scalar::from(n as u16));
        assert_eq!(element, Scalar::from(n as u32));
        assert_eq!(element, Scalar::from(n as u64));
        assert_eq!(element, Scalar::from(n as u128));
    }
}
