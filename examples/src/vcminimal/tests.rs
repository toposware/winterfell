// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn vc_minimal_test_basic_proof_verification() {
    let vc = Box::new(super::VCMinimalExample::new(build_options(false), 2, 8));
    crate::tests::test_basic_proof_verification(vc);
}

#[test]
fn vc_minimal_test_basic_proof_verification_extension() {
    let vc = Box::new(super::VCMinimalExample::new(build_options(true), 2, 8));
    crate::tests::test_basic_proof_verification(vc);
}

#[test]

fn vc_minimal_test_basic_proof_verification_fail() {
    let vc = Box::new(super::VCMinimalExample::new(build_options(false), 2, 8));
    crate::tests::test_basic_proof_verification_fail(vc);
}

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(3, 2, 0, HashFunction::Blake3_256, extension, 4, 256)
}