// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::air::Assertion;
use core::fmt::{Display, Formatter};
use math::{log2, FieldElement, StarkField};
use utils::collections::Vec;

// CONSTRAINT DIVISOR
// ================================================================================================
/// The denominator portion of boundary and transition constraints.
///
/// A divisor is described by a combination of a sparse polynomial, which describes the numerator
/// of the divisor and a set of exemption points, which describe the denominator of the divisor.
/// The numerator polynomial is described as multiplication of tuples where each tuple encodes
/// an expression $(x^a - b)$. The exemption points encode expressions $(x - a)$.
///
/// For example divisor $(x^a - 1) \cdot (x^b - 2) / (x - 3)$ can be represented as:
/// numerator: `[(a, 1), (b, 2)]`, exemptions: `[3]`.
///
/// A divisor cannot be instantiated directly, and instead must be created either for an
/// [Assertion] or for a transition constraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDivisor<B: StarkField> {
    pub(super) numerator: Vec<(usize, B)>,
    pub(super) exemptions: Vec<B>,
}

impl<B: StarkField> ConstraintDivisor<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new divisor instantiated from the provided parameters.
    fn new(numerator: Vec<(usize, B)>, exemptions: Vec<B>) -> Self {
        ConstraintDivisor {
            numerator,
            exemptions,
        }
    }

    /// Builds a divisor for transition constraints.
    ///
    /// The divisor is described by a vector of tuples of the form (step, offset, exemptions).
    /// Each tuple describes that the constraint should be satisied with a period of trace_length/step
    /// after shifting by offset and excluding the last exemptions steps.
    ///
    /// For transition constraints, the divisor polynomial $z(x)$ is
    /// $z(x) = \prod_{i=1}^{\ell} z_i(x)$ where
    ///
    /// $$
    /// z_i(x) = \frac{x^{n_i} - g^{n_i*o_i}}{ \prod_{e} (x - g^{e})}
    /// $$
    ///
    /// where, $n$ is the length of the execution trace, $g$ is the generator of the trace,
    /// n_i is the number of frames checked by the divisor, o_i the offset where we do these checks  
    /// and e is a set of exemptions for each term z_i(x).
    ///
    /// The exemption points are defined by taking the last exemptions elements that were supposed to
    /// be checked.
    pub fn from_transition<E>(divisor: &[(usize, usize, usize)], trace_length: usize) -> Self
    where
        E: FieldElement<BaseField = B>,
    {
        let mut steps = Vec::new();
        let mut exemptions = Vec::new();

        for numerator in divisor {
            let &(step, offset, num_exemptions) = numerator;
            steps.push((
                step,
                get_trace_domain_value_at::<B>(trace_length, step * offset),
            ));
            let e: Vec<_> = (step - num_exemptions..step)
                .map(|s| {
                    get_trace_domain_value_at::<B>(trace_length, (trace_length / step) * s + offset)
                })
                .collect();
            exemptions.extend(e);
        }
        Self::new(steps, exemptions)
    }

    /// Builds the vanishing polynomial of the trace domain: $z(x) = x^n-1$.
    pub fn from_default_numerator(trace_length: usize) -> Self {
        Self::new(vec![(trace_length, B::ONE)], vec![])
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
        if assertion.first_step == 0 {
            Self::new(vec![(num_steps, B::ONE)], vec![])
        } else {
            let trace_offset = num_steps * assertion.first_step;
            let offset = get_trace_domain_value_at::<B>(trace_length, trace_offset);
            Self::new(vec![(num_steps, offset)], vec![])
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the numerator portion of this constraint divisor.
    pub fn numerator(&self) -> &[(usize, B)] {
        &self.numerator
    }

    /// Returns exemption points (the denominator portion) of this constraints divisor.
    pub fn exemptions(&self) -> &[B] {
        &self.exemptions
    }

    /// Returns the degree of the divisor polynomial
    pub fn degree(&self) -> usize {
        let numerator_degree = self
            .numerator
            .iter()
            .fold(0, |degree, term| degree + term.0);
        let denominator_degree = self.exemptions.len();
        numerator_degree - denominator_degree
    }

    // EVALUATORS
    // --------------------------------------------------------------------------------------------
    /// Evaluates the divisor polynomial at the provided `x` coordinate.
    pub fn evaluate_at<E: FieldElement<BaseField = B>>(&self, x: E) -> E {
        // compute the numerator value
        let mut numerator = E::ONE;
        for (degree, constant) in self.numerator.iter() {
            let v = x.exp((*degree as u32).into());
            let v = v - E::from(*constant);
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
        self.exemptions
            .iter()
            .fold(E::ONE, |r, &e| r * (x - E::from(e)))
    }

    // Decomposition
    // --------------------------------------------------------------------------------------------

    /// Express a divisor of the form $x^k - g^{j\cdot k}$ as a divisor of the polynomial $x^n - 1$ where
    /// $n$ is the trace length. Concretely, we need to compute the polynomial
    ///
    /// $$
    /// D'(x) = \prod_{i=0,i\neq j}^{n/k-1} (x^k - g^{i\cdot k})
    /// $$
    pub fn decompose_single<E: FieldElement<BaseField = B>>(&self, trace_length: usize) -> Self {
        assert_eq!(
            self.numerator().len(),
            1,
            "numerator length should be 1 but was {}",
            self.numerator().len()
        );

        let mut numerator: Vec<(usize, B)> = Vec::new();
        // The number of vanishing points of a divisor
        let k = self.numerator()[0].0;
        // subgroup_generator
        let g = B::get_root_of_unity(trace_length.trailing_zeros());
        // create a new divisor that contains all the points except the ones defined by the given
        // numerator
        for i in 0..trace_length / k {
            let h = g.exp(((i * k) as u64).into());
            if h != self.numerator()[0].1 {
                numerator.push((k, h));
            }
        }
        Self::new(numerator, Vec::new())
    }

    /// Express a divisor of the form $\Prod_k x^k - g^{j\cdot k}$ as a vector of divisors.
    /// Each element corresponds to a value k.
    /// Note that summing the divisors yields the polynomial
    /// $m(x^n - 1)$ where m is the elements in the numerator of the divisor polynomial
    pub fn decompose<E: FieldElement<BaseField = B>>(&self, trace_length: usize) -> Vec<Self> {
        let decomposition = self
            .numerator()
            .iter()
            .map(|numerator| {
                ConstraintDivisor::new(vec![*numerator], vec![]).decompose_single::<B>(trace_length)
            })
            .collect::<Vec<_>>();
        decomposition
    }

    /// Evaluates a decomposed divisor at $x$. Note that if the divisor is $d(x)$, the returned
    /// value is $\frac{x^n-1}{d(x)}$
    pub fn evaluate_decomposition<E: FieldElement<BaseField = B>>(
        &self,
        trace_length: usize,
        x: E,
    ) -> E {
        // evaluate each of the decomposed divisors
        let individual_evaluations = self
            .decompose::<E>(trace_length)
            .iter()
            .map(|d| d.evaluate_at(x))
            .collect::<Vec<_>>();

        individual_evaluations
            .iter()
            .fold(E::ZERO, |acc, e| acc + *e)
    }

    // ASSOCIATED FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Evaluates the vanishing polynomial of the trace domain $x^n-1$ at $x$.
    pub fn evaluate_default_numerator<E: FieldElement<BaseField = B>>(
        trace_length: usize,
        x: E,
    ) -> E {
        x.exp((trace_length as u64).into()) - E::ONE
    }
}

impl<B: StarkField> Display for ConstraintDivisor<B> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        for (degree, offset) in self.numerator.iter() {
            write!(f, "(x^{} - {})", degree, offset)?;
        }
        if !self.exemptions.is_empty() {
            write!(f, " / ")?;
            for x in self.exemptions.iter() {
                write!(f, "(x - {})", x)?;
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

    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn constraint_divisor_degree() {
        // single term numerator
        let div = ConstraintDivisor::new(vec![(4, BaseElement::ONE)], vec![]);
        assert_eq!(4, div.degree());

        // multi-term numerator
        let div = ConstraintDivisor::new(
            vec![
                (4, BaseElement::ONE),
                (2, BaseElement::new(2)),
                (3, BaseElement::new(3)),
            ],
            vec![],
        );
        assert_eq!(9, div.degree());

        // multi-term numerator with exemption points
        let div = ConstraintDivisor::new(
            vec![
                (4, BaseElement::ONE),
                (2, BaseElement::new(2)),
                (3, BaseElement::new(3)),
            ],
            vec![BaseElement::ONE, BaseElement::new(2)],
        );
        assert_eq!(7, div.degree());
    }

    #[test]
    fn constraint_divisor_evaluation() {
        // single term numerator: (x^4 - 1)
        let div = ConstraintDivisor::new(vec![(4, BaseElement::ONE)], vec![]);
        assert_eq!(BaseElement::new(15), div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator: (x^4 - 1) * (x^2 - 2) * (x^3 - 3)
        let div = ConstraintDivisor::new(
            vec![
                (4, BaseElement::ONE),
                (2, BaseElement::new(2)),
                (3, BaseElement::new(3)),
            ],
            vec![],
        );
        let expected = BaseElement::new(15) * BaseElement::new(2) * BaseElement::new(5);
        assert_eq!(expected, div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator with exemption points:
        // (x^4 - 1) * (x^2 - 2) * (x^3 - 3) / ((x - 1) * (x - 2))
        let div = ConstraintDivisor::new(
            vec![
                (4, BaseElement::ONE),
                (2, BaseElement::new(2)),
                (3, BaseElement::new(3)),
            ],
            vec![BaseElement::ONE, BaseElement::new(2)],
        );
        let expected = BaseElement::new(255) * BaseElement::new(14) * BaseElement::new(61)
            / BaseElement::new(6);
        assert_eq!(expected, div.evaluate_at(BaseElement::new(4)));
    }

    #[test]
    fn constraint_divisor_equivalence() {
        let n = 8_usize;
        let g = BaseElement::get_root_of_unity(n.trailing_zeros());
        let k = 4 as u32;
        let j = n as u32 / k;

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
            ConstraintDivisor::new(vec![(k as usize, g.exp(k.into()))], vec![]),
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
            ConstraintDivisor::new(vec![(k as usize, g.exp((offset * k).into()))], vec![]),
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

    #[test]
    fn divisor_decomposition() {
        let n = 128_usize;
        let g = BaseElement::get_root_of_unity(n.trailing_zeros());

        // Assert divisor decomposition is valid. Product of decomposed values and given divisor
        // should be equal to the evaluation of X^n-1
        let vanishing_polynomial =
            ConstraintDivisor::new(vec![(n as usize, BaseElement::ONE)], vec![]);

        // sample a random point for checking polynomial relations
        let value = rand_utils::rand_value::<BaseElement>();

        // 1. plain (single numerator element) decomposition
        // subgroups
        for k in [1, 2, 4, 8, 16, 32, 64, 128] {
            // offsets
            for j in 0..n / k {
                // create a divisor of period n/k and offset j
                let divisor = ConstraintDivisor::new(
                    vec![(k as usize, g.exp((j as u32 * k as u32).into()))],
                    vec![],
                );
                let decomposed_divisor = divisor.decompose_single::<BaseElement>(n);

                let expected_evaluation = vanishing_polynomial.evaluate_at(value);
                let actual_evaluation =
                    decomposed_divisor.evaluate_at(value) * divisor.evaluate_at(value);
                assert_eq!(expected_evaluation, actual_evaluation);
            }
        }

        // 2. full decomposition
        // sample 5 random divisors
        let mut rng = thread_rng();
        let mut numerators = vec![];
        // choose 10 random divisor numerators
        for _ in 0..10 {
            let size = 2usize.pow(rng.gen_range(0..8));
            // period one implies offset == 0
            if n / size == 1 {
                numerators.push((size, g.exp((size as u32 * 0 as u32).into())));
            } else {
                let offset = rng.gen_range(0..n / size);
                numerators.push((size, g.exp((size as u32 * offset as u32).into())));
            }
        }
        // get the separate divisors defining the numerator

        let divisor = ConstraintDivisor::new(numerators, vec![]);
        let divisors = divisor
            .numerator()
            .iter()
            .map(|num| ConstraintDivisor::new(vec![*num], vec![]))
            .collect::<Vec<_>>();
        // get divisor decomposition
        let decomposed_divisor = divisor.decompose::<BaseElement>(n);
        // get expected evaluation. This should be l(x^n-1) where l is the divisor numerator length
        let expected_evaluation = BaseElement::new(divisor.numerator().len() as u128)
            * vanishing_polynomial.evaluate_at(value);

        // evaluate each of the decomposed divisors
        let individual_evaluations = decomposed_divisor
            .iter()
            .enumerate()
            .map(|(i, d)| d.evaluate_at(value) * divisors[i].evaluate_at(value))
            .collect::<Vec<_>>();
        let actual_evaluation = individual_evaluations
            .iter()
            .fold(BaseElement::ZERO, |acc, e| acc + *e);
        assert_eq!(expected_evaluation, actual_evaluation);

        // 3. default divisor numerator computation
        let expected_evaluation =
            ConstraintDivisor::new(vec![(n, BaseElement::ONE)], vec![]).evaluate_at(value);
        let actual_evaluation = ConstraintDivisor::evaluate_default_numerator(n, value);
        assert_eq!(expected_evaluation, actual_evaluation);
    }
}
