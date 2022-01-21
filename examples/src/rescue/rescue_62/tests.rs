// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn rescue_test_basic_proof_verification() {
    let rescue_eg = Box::new(super::RescueExample::new(128, build_options()));
    crate::tests::test_basic_proof_verification(rescue_eg);
}

#[test]
fn rescue_test_basic_proof_verification_fail() {
    let rescue_eg = Box::new(super::RescueExample::new(128, build_options()));
    crate::tests::test_basic_proof_verification_fail(rescue_eg);
}

fn build_options() -> ProofOptions {
    ProofOptions::new(
        42,
        4,
        0,
        HashFunction::Blake3_256,
        FieldExtension::None,
        4,
        256,
    )
}
