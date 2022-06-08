// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::super::utils::build_proof_options;

#[test]
fn cairo_test_basic_proof_verification() {
    let cairo = Box::new(super::CairoExample::new(build_proof_options(false)));
    crate::tests::test_basic_proof_verification(cairo);
}

#[test]
fn cairo_test_basic_proof_verification_extension() {
    let cairo = Box::new(super::CairoExample::new(build_proof_options(true)));
    crate::tests::test_basic_proof_verification(cairo);
}

#[test]
fn cairo_test_basic_proof_verification_fail() {
    let cairo = Box::new(super::CairoExample::new(build_proof_options(false)));
    crate::tests::test_basic_proof_verification_fail(cairo);
}
