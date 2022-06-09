// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn degree_problem_basic_proof_verification() {
    let cairo_cpu = Box::new(super::DegreeProblemExample::new(build_options(false)));
    crate::tests::test_basic_proof_verification(cairo_cpu);
}

// #[test]
// fn cairo_cpu_test_basic_proof_verification_extension() {
//     let cairo_cpu = Box::new(super::DegreeProblemExample::new(build_proof_options(true)));
//     crate::tests::test_basic_proof_verification(cairo_cpu);
// }

// #[test]

// fn cairo_cpu_test_basic_proof_verification_fail() {
//     let cairo_cpu = Box::new(super::DegreeProblemExample::new(build_proof_options(false)));
//     crate::tests::test_basic_proof_verification_fail(cairo_cpu);
// }

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(28, 4, 0, HashFunction::Blake3_256, extension, 4, 256)
}