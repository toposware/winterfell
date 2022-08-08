// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn pedersen_cpu_test_basic_proof_verification() {
    let pedersen_hash = Box::new(super::PedersenHashExample::new(build_options(false)));
    crate::tests::test_basic_proof_verification(pedersen_hash);
}

#[test]
fn pedersen_cpu_test_basic_proof_verification_extension() {
    let pedersen_hash = Box::new(super::PedersenHashExample::new(build_options(true)));
    crate::tests::test_basic_proof_verification(pedersen_hash);
}

// Test commented: currently cairo does not admit any inputs
// #[test]

// fn pedersen_hash_test_basic_proof_verification_fail() {
//     let pedersen_hash = Box::new(super::PedersenHashExample::new(build_options(false)));
//     crate::tests::test_basic_proof_verification_fail(pedersen_hash);
// }

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(28, 4, 0, HashFunction::Blake3_256, extension, 4, 256)
}