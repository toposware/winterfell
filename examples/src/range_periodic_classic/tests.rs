// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

#[cfg(test)]
pub fn build_proof_options(use_extension_field: bool) -> winterfell::ProofOptions {
    use winterfell::{FieldExtension, HashFunction, ProofOptions};

    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(28, 4, 0, HashFunction::Blake3_256, extension, 4, 256)
}

#[test]
fn range_divisors_test_basic_proof_verification() {
    let range = Box::new(super::RangeExample::new(64, build_proof_options(false)));
    crate::tests::test_basic_proof_verification(range);
}

#[test]
fn range_divisors_test_basic_proof_verification_extension() {
    let collatz = Box::new(super::RangeExample::new(64, build_proof_options(true)));
    crate::tests::test_basic_proof_verification(collatz);
}

#[test]
fn collatz_divisors_test_basic_proof_verification_fail() {
    let collatz = Box::new(super::RangeExample::new(64, build_proof_options(false)));
    crate::tests::test_basic_proof_verification_fail(collatz);
}
