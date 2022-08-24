// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::air::Assertion;
use core::fmt::{Display, Formatter};
use math::{log2, FieldElement, StarkField};
use utils::collections::Vec;

// CONSTRAINT DIVISOR PRODUCT
// ================================================================================================
/// The building block of a divisor. It expresses sparse polynomials of the form $(X^k - g^b)$.
/// The term $k$ (subgroup) determines the number of elements on which constraints are
/// applied/excluded and $g^b$ defines an offset. $g$ is the trace domain generator.
/// When $b=k*j$ the product applies/excludes the constraints at elements $n/k+j, 2n/k+j,\ldots$.
///
/// The product is defined by the number of elements it involves (subgroup), the coset elements
/// $h=g^b$ and its dlog $b$ w.r.t. the trace domain generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDivisorProduct<B: StarkField> {
    pub(super) subgroup: usize,
    pub(super) coset_dlog: usize,
    pub(super) coset_elem: B,
}

impl<B: StarkField> ConstraintDivisorProduct<B> {
    /// Returns a new divisor product. Given a trace_length, a period and an offset
    /// it returns the product that involves elements in the trace that are period far apart
    /// with first element being offset.
    fn new(trace_length: usize, period: usize, offset: usize) -> Self {
        // TODO [divisors]: Assertions:
        //      1. trace_length is a power of 2
        //      2. period is a power of 2
        //      3. period < trace_length
        //      3. 0 <= offset < period
        let subgroup = trace_length / period;
        ConstraintDivisorProduct {
            subgroup,
            coset_dlog: subgroup * offset,
            coset_elem: get_trace_domain_value_at::<B>(trace_length, subgroup * offset),
        }
    }

    /// Returns the number of points the product involves.
    pub fn subgroup(&self) -> usize {
        self.subgroup
    }

    /// Returns the dlog of the coset element w.r.t. the trace domain.
    pub fn coset_dlog(&self) -> usize {
        self.coset_dlog
    }

    /// Returns the coset element of the product term.
    pub fn coset_elem(&self) -> B {
        self.coset_elem
    }

    /// Returns the degree of the sparse polynomial defined by the divisor product.
    /// Note this is equal to subgroup
    pub fn degree(&self) -> usize {
        self.subgroup
    }
}

// CONSTRAINT DIVISOR
// ================================================================================================
/// The denominator portion of boundary and transition constraints.
///
/// A divisor is described by a set of [ConstraintDivisorProducts]. The numerator of the divisor
/// defines the points in the trace where a constraint applies and the denominator portion excludes
/// points.
///
/// A divisor cannot be instantiated directly, and instead must be created either for an
/// [Assertion] or for a transition constraint.

// TODO [divisors]: add docs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDivisor<B: StarkField> {
    pub(super) numerator: Vec<ConstraintDivisorProduct<B>>,
    pub(super) denominator: Vec<ConstraintDivisorProduct<B>>,
}

impl<B: StarkField> ConstraintDivisor<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new divisor instantiated from the provided parameters.
    fn new(
        numerator: Vec<ConstraintDivisorProduct<B>>,
        denominator: Vec<ConstraintDivisorProduct<B>>,
    ) -> Self {
        ConstraintDivisor {
            numerator,
            denominator,
        }
    }

    /// Builds a divisor for transition constraints.
    ///
    /// Takes as input a tuple of vectors representing products.
    ///
    /// The first vector determines the points where the transition should hold.
    /// Concretely, the vector is of the form (period, offset, num_exemptions)
    /// and forces the constraint to hold for all steps in every period steps
    /// starting from offset and excluding the last num_exemptions steps.
    ///
    /// The second vector determines (complex) exemption points. Its element
    /// is of the form (period, offset) and exempts all elements in steps
    /// $offset, offset + period, offset + 2period,\ldots$ from being asserted.
    pub fn from_transition(
        trace_length: usize,
        divisor: &(Vec<(usize, usize, usize)>, Vec<(usize, usize)>),
    ) -> Self {
        // TODO [divisors]: add assertions:

        // Build numerator product terms
        let numerator: Vec<ConstraintDivisorProduct<B>> = divisor
            .0
            .iter()
            .map(|(period, offset, _)| {
                ConstraintDivisorProduct::new(trace_length, *period, *offset)
            })
            .collect();

        // Build denominator product terms. Here we exclude points defined in the
        // second element of divisor as well as any last step exemptions defined in
        // the first term.
        let mut denominator: Vec<ConstraintDivisorProduct<B>> = vec![];
        for (period, offset, num_exemptions) in divisor.0.iter() {
            let exemptions = (1..=*num_exemptions)
                .map(|step| {
                    ConstraintDivisorProduct::new(
                        trace_length,
                        trace_length,
                        (trace_length / period - step) * period + offset,
                    )
                })
                .collect::<Vec<_>>();
            denominator.extend(exemptions);
        }

        let complex_exemptions: Vec<ConstraintDivisorProduct<B>> = divisor
            .1
            .iter()
            .map(|(period, offset)| ConstraintDivisorProduct::new(trace_length, *period, *offset))
            .collect();
        denominator.extend(complex_exemptions);

        Self::new(numerator, denominator)
    }

    /// Builds a divisor for a boundary constraint described by the assertion.
    ///
    /// For boundary constraints, the divisor polynomial is defined as:
    ///
    /// $$
    /// z(x) = x^k - g^{a \cdot k}
    /// $$
    ///
    /// where $g$ is the generator of the trace domain, $k$ is the number of asserted steps, and
    /// $a$ is the step offset in the trace domain. Specifically:
    /// * For an assertion against a single step, the polynomial is $(x - g^a)$, where $a$ is the
    ///   step on which the assertion should hold.
    /// * For an assertion against a sequence of steps which fall on powers of two, it is
    ///   $(x^k - 1)$ where $k$ is the number of asserted steps.
    /// * For assertions against a sequence of steps which repeat with a period that is a power
    ///   of two but don't fall exactly on steps which are powers of two (e.g. 1, 9, 17, ... )
    ///   it is $(x^k - g^{a \cdot k})$, where $a$ is the number of steps by which the assertion steps
    ///   deviate from a power of two, and $k$ is the number of asserted steps. This is equivalent to
    ///   $(x - g^a) \cdot (x - g^{a + j}) \cdot (x - g^{a + 2 \cdot j}) ... (x - g^{a + (k  - 1) \cdot j})$,
    ///   where $j$ is the length of interval between asserted steps (e.g. 8).
    ///
    /// # Panics
    /// Panics of the specified `trace_length` is inconsistent with the specified `assertion`.
    pub fn from_assertion<E>(assertion: &Assertion<E>, trace_length: usize) -> Self
    where
        E: FieldElement<BaseField = B>,
    {
        let num_steps = assertion.get_num_steps(trace_length);
        let numerator = ConstraintDivisorProduct::new(
            trace_length,
            trace_length / num_steps,
            assertion.first_step,
        );
        Self::new(vec![numerator], vec![])
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the numerator portion of this constraint divisor.
    pub fn numerator(&self) -> &[ConstraintDivisorProduct<B>] {
        &self.numerator
    }

    /// Returns exemption points (the denominator portion) of this constraints divisor.
    pub fn denominator(&self) -> &[ConstraintDivisorProduct<B>] {
        &self.denominator
    }

    /// Returns the degree of the divisor polynomial
    pub fn degree(&self) -> usize {
        let numerator_degree = self
            .numerator
            .iter()
            .fold(0, |degree, term| degree + term.degree());
        let denominator_degree = self
            .denominator
            .iter()
            .fold(0, |degree, term| degree + term.degree());
        numerator_degree - denominator_degree
    }

    // EVALUATORS
    // --------------------------------------------------------------------------------------------
    /// Evaluates the divisor polynomial at the provided `x` coordinate.
    pub fn evaluate_at<E: FieldElement<BaseField = B>>(&self, x: E) -> E {
        // compute the numerator value
        let mut numerator = E::ONE;
        for product in self.numerator.iter() {
            let v = x.exp((product.degree() as u32).into()) - E::from(product.coset_elem);
            numerator *= v;
        }

        // compute the denominator value
        let denominator = self.evaluate_exemptions_at(x);

        numerator / denominator
    }

    /// Evaluates the denominator of this divisor (the exemption points) at the provided `x`
    /// coordinate.
    #[inline(always)]
    pub fn evaluate_exemptions_at<E: FieldElement<BaseField = B>>(&self, x: E) -> E {
        let mut denominator = E::ONE;
        for product in self.denominator.iter() {
            let v = x.exp((product.degree() as u32).into()) - E::from(product.coset_elem);
            denominator *= v;
        }
        denominator
    }
}

impl<B: StarkField> Display for ConstraintDivisor<B> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        for product in self.numerator.iter() {
            write!(f, "(x^{} - {})", product.degree(), product.coset_elem())?;
        }
        if self.denominator.is_empty() {
            write!(f, " / ")?;
            for product in self.denominator.iter() {
                write!(f, "(x^{} - {})", product.degree(), product.coset_elem())?;
            }
        }
        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Returns g^step, where g is the generator of trace domain.
pub fn get_trace_domain_value_at<B: StarkField>(trace_length: usize, step: usize) -> B {
    debug_assert!(
        step < trace_length,
        "step must be in the trace domain [0, {})",
        trace_length
    );
    let g = B::get_root_of_unity(log2(trace_length));
    g.exp((step as u64).into())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use math::{fields::f128::BaseElement, polynom};

    #[test]
    fn constraint_divisor_degree() {
        // single term numerator
        let div = ConstraintDivisor::new(
            vec![ConstraintDivisorProduct::<BaseElement>::new(16, 4, 0)],
            vec![],
        );
        assert_eq!(4, div.degree());

        // multi-term numerator
        let div = ConstraintDivisor::new(
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 4, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 2, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
            vec![],
        );
        assert_eq!(14, div.degree());

        // multi-term numerator with exemption points
        let div = ConstraintDivisor::new(
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 4, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 2, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 16, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
        );
        assert_eq!(14 - 5, div.degree());
    }

    #[test]
    fn constraint_divisor_evaluation() {
        // single term numerator: (x^4 - 1)
        let div = ConstraintDivisor::new(
            vec![ConstraintDivisorProduct::<BaseElement>::new(16, 4, 0)],
            vec![],
        );
        assert_eq!(BaseElement::new(15), div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator: (x^4 - g^4) * (x^8 - 1) * (x^2 - g^6)
        let div = ConstraintDivisor::new(
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 4, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 2, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
            vec![],
        );
        let g_trace = BaseElement::get_root_of_unity(16_usize.trailing_zeros());
        let expected = (BaseElement::new(16) - g_trace.exp(4))
            * BaseElement::new(255)
            * (BaseElement::new(4) - g_trace.exp(2 * 3));
        assert_eq!(expected, div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator with exemption points:
        // (x^4 - g^4) * (x^8 - 1) * (x^3 - g^6) / (x^2 - g^2) (x-1) (x^2-g^6)
        let div = ConstraintDivisor::new(
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 4, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 2, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
            vec![
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 1),
                ConstraintDivisorProduct::<BaseElement>::new(16, 16, 0),
                ConstraintDivisorProduct::<BaseElement>::new(16, 8, 3),
            ],
        );
        let expected_numerator = (BaseElement::new(16) - g_trace.exp(4))
            * BaseElement::new(255)
            * (BaseElement::new(4) - g_trace.exp(2 * 3));
        let expected_denominator =
            (BaseElement::new(4) - g_trace.exp(2)) * (BaseElement::new(4) - g_trace.exp(6));
        assert_eq!(
            expected_numerator / expected_denominator,
            div.evaluate_at(BaseElement::new(2))
        );
    }

    #[test]
    fn constraint_divisor_equivalence() {
        let n = 8_usize;
        let g = BaseElement::get_root_of_unity(n.trailing_zeros());
        let k = 4 as u32;
        let j = n as u32 / k; // period

        // ----- periodic assertion divisor, no offset --------------------------------------------

        // create a divisor for assertion which repeats every 2 steps starting at step 0
        let assertion = Assertion::periodic(0, 0, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);

        // z(x) = x^4 - 1 = (x - 1) * (x - g^2) * (x - g^4) * (x - g^6)
        let poly = polynom::mul(
            &polynom::mul(
                &[-BaseElement::ONE, BaseElement::ONE],
                &[-g.exp(j.into()), BaseElement::ONE],
            ),
            &polynom::mul(
                &[-g.exp((2 * j).into()), BaseElement::ONE],
                &[-g.exp((3 * j).into()), BaseElement::ONE],
            ),
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i % (j as usize) == 0 {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }

        // ----- periodic assertion divisor, with offset ------------------------------------------

        // create a divisor for assertion which repeats every 2 steps starting at step 1
        let offset = 1u32;
        let assertion = Assertion::periodic(0, offset as usize, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);
        assert_eq!(
            ConstraintDivisor::new(
                vec![ConstraintDivisorProduct::new(
                    n,
                    j as usize,
                    offset as usize
                )],
                vec![]
            ),
            divisor
        );

        // z(x) = x^4 - g^4 = (x - g) * (x - g^3) * (x - g^5) * (x - g^7)
        let poly = polynom::mul(
            &polynom::mul(
                &[-g.exp(offset.into()), BaseElement::ONE],
                &[-g.exp((offset + j).into()), BaseElement::ONE],
            ),
            &polynom::mul(
                &[-g.exp((offset + 2 * j).into()), BaseElement::ONE],
                &[-g.exp((offset + 3 * j).into()), BaseElement::ONE],
            ),
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i % (j as usize) == offset as usize {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }

        // create a divisor for assertion which repeats every 4 steps starting at step 3
        let offset = 3u32;
        let k = 2 as u32;
        let j = n as u32 / k;
        let assertion = Assertion::periodic(0, offset as usize, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);
        assert_eq!(
            ConstraintDivisor::new(
                vec![ConstraintDivisorProduct::new(
                    n,
                    j as usize,
                    offset as usize
                )],
                vec![]
            ),
            divisor
        );

        // z(x) = x^2 - g^6 = (x - g^3) * (x - g^7)
        let poly = polynom::mul(
            &[-g.exp(offset.into()), BaseElement::ONE],
            &[-g.exp((offset + j).into()), BaseElement::ONE],
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i % (j as usize) == offset as usize {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }
    }
}
