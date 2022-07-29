// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn divisors_exemptions_test_basic_proof_verification() {
    let div = Box::new(super::DivisorsExemptionsExample::new(
        [16, 5],
        build_options(false),
    ));
    crate::tests::test_basic_proof_verification(div);
}

#[test]

fn divisors_exemptions_test_basic_proof_verification_extension() {
    let div = Box::new(super::DivisorsExemptionsExample::new(
        [16, 5],
        build_options(true),
    ));
    crate::tests::test_basic_proof_verification(div);
}

#[test]
fn divisors_exemptions_test_basic_proof_verification_fail() {
    let div = Box::new(super::DivisorsExemptionsExample::new(
        [16, 5],
        build_options(false),
    ));
    crate::tests::test_basic_proof_verification_fail(div);
}

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(5, 2, 0, HashFunction::Blake3_256, extension, 4, 256)
}
