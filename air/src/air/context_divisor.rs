// AIR CONTEXT DIVISOR
// ================================================================================================
/// Description of the divisor used by AIR Context. The divisor is described by the numerator and
/// the denominator.
///
/// The numerator is a vector each containing the period where the divisor should
/// force the constraint, the offset and the number of exemptions describing the final points
/// the divisor should exclude.
///
/// The denominator describes complex exemptions, points where the constraint should not hold.
/// It is defined similarly with the period and the offset but does not have exemptions.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextDivisor {
  pub(super) numerator: Vec<(usize, usize, usize)>,
  pub(super) denominator: Vec<(usize, usize)>,
}

impl ContextDivisor {
  // CONSTRUCTORS
  // --------------------------------------------------------------------------------------------
  /// Returns a new instance of [Divisor] by getting the numerator and denominator vector
  /// as inputs
  pub fn new(numerator: Vec<(usize, usize, usize)>, denominator: Vec<(usize, usize)>) -> Self {
    ContextDivisor {
      numerator,
      denominator,
    }
  }

  /// Returns a new instance of the default divisor that checks the constraint everywhere apart
  /// from the last step
  pub fn default() -> Self {
    ContextDivisor {
      numerator: vec![(1, 0, 1)],
      denominator: vec![],
    }
  }

  // PUBLIC ACCESSORS
  // --------------------------------------------------------------------------------------------

  /// Returns the numerator vectors.
  pub fn numerator(&self) -> &[(usize, usize, usize)] {
    &self.numerator
  }

  /// Returns the denominator vectors.
  pub fn denominator(&self) -> &[(usize, usize)] {
    &self.denominator
  }
}
