// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn subset_sum_test_basic_proof_verification() {
    let subset_sum = Box::new(super::SubsetSumExample::new(build_options(false)));
    crate::tests::test_basic_proof_verification(subset_sum);
}

#[test]
fn subset_sum_test_basic_proof_verification_extension() {
    let subset_sum = Box::new(super::SubsetSumExample::new(build_options(true)));
    crate::tests::test_basic_proof_verification(subset_sum);
}

#[test]
fn mux_basic_proof_verification() {
    let mux = Box::new(super::MuxExample::new(build_options(false)));
    crate::tests::test_basic_proof_verification(mux);
}

// Test commented: currently cairo does not admit any inputs
// #[test]

// fn subset_sum_test_basic_proof_verification_fail() {
//     let subset_sum = Box::new(super::SubsetSumExample::new(build_options(false)));
//     crate::tests::test_basic_proof_verification_fail(subset_sum);
// }

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(28, 4, 0, HashFunction::Blake3_256, extension, 4, 256)
}