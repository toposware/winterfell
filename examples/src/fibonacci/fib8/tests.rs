// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2023 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::super::utils::build_proof_options;

#[test]
fn fib8_test_basic_proof_verification() {
    let fib = Box::new(super::Fib8Example::new(64, build_proof_options(false)));
    crate::tests::test_basic_proof_verification(fib);
}

#[test]
fn fib8_test_basic_proof_verification_extension() {
    let fib = Box::new(super::Fib8Example::new(64, build_proof_options(true)));
    crate::tests::test_basic_proof_verification(fib);
}

#[test]
fn fib8_test_basic_proof_verification_fail() {
    let fib = Box::new(super::Fib8Example::new(64, build_proof_options(false)));
    crate::tests::test_basic_proof_verification_fail(fib);
}
