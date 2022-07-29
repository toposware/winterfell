// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use examples::{vcminimal, Example};
use winterfell::{FieldExtension, HashFunction, ProofOptions};

const SIZES: [usize; 3] = [16_384, 65_536, 262_144];
const WIDTHS: [usize; 3] = [32, 64, 128];

mod perf;

fn vcminimal(c: &mut Criterion) {
    let mut group = c.benchmark_group("vcminimal");
    group.sample_size(10);

    let options = ProofOptions::new(
        108,
        2,
        22,
        HashFunction::Blake3_256,
        FieldExtension::Quadratic,
        4,
        256,
    );

    // prover
    for &size in SIZES.iter() {
        for &width in WIDTHS.iter() {
            let max_pow = (width as f64).log2().ceil() as u32;
            for real_width in (1u32..=max_pow).map(|i| 2usize.pow(i)) {
                let vc = vcminimal::VCMinimalExample::new(options.clone(), 2, size, width, real_width);
                group.bench_function(
                    BenchmarkId::from_parameter(format!("prover{:?}", (size, width, real_width))),
                    |bench| {
                        bench.iter_with_large_setup(
                            || vc.get_trace_and_prover(),
                            |(trace, prover)| {
                                vc.prove_with_precomputed_trace_and_prover(trace, prover)
                            }
                        );
                    },
                );
            }
        }
    }

    // verifier
    for &size in SIZES.iter() {
        for &width in WIDTHS.iter() {
            let max_pow = (width as f64).log2().ceil() as u32;
            for real_width in (1u32..=max_pow).map(|i| 2usize.pow(i)) {
                let vc = vcminimal::VCMinimalExample::new(options.clone(), 2, size, width, real_width);

                group.bench_function(
                    BenchmarkId::from_parameter(format!("verifier{:?}", (size, width, real_width))),
                    |bench| {
                        bench.iter_with_large_setup(|| vc.prove(), |proof| vc.verify(proof));
                    },
                );

                let proof = vc.prove();
                println!(
                    "\t Proof size: {:.1} KB ({} bits security)",
                    proof.to_bytes().len() as f64 / 1024f64,
                    proof.security_level(true),
                );
                for (bytes, name) in proof.to_byte_vec().iter().zip(
                    ["context:", "commitments", "trace queries", "constraint queries:", "ood frame", "fri proof", "pow nonce"].into_iter()
                ) {
                    println!("\t\t{}: {:.1} KB", name, bytes.len() as f64 / 1024f64);
                }
            }
        }
    }
    group.finish();
}

criterion_group!(
    name = vcminimal_group;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = vcminimal
);
criterion_main!(vcminimal_group);
