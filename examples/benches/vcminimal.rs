// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use examples::{vcminimal, Example};
use std::time::Duration;
use winterfell::{FieldExtension, HashFunction, ProofOptions};

const SIZES: [usize; 1] = [64]; //[16_384, 65_536, 262_144];
const WIDTHS: [usize; 3] = [32, 64, 128];

fn vcminimal(c: &mut Criterion) {
    let mut group = c.benchmark_group("vcminimal");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    let options = ProofOptions::new(
        32,
        8,
        0,
        HashFunction::Blake3_256,
        FieldExtension::None,
        4,
        256,
    );
    // prover
    for &size in SIZES.iter(){
        for &width in WIDTHS.iter(){
            let max_pow = (width as f64).log2().ceil() as u32;
            for real_width in (1u32..max_pow).map(|i| 2usize.pow(i)) 
            {
                let vc = vcminimal::VCMinimalExample::new(
                    options.clone(),
                    2,
                    size,
                    width,
                    16
                );
                group.bench_function(
                    BenchmarkId::from_parameter(format!("prover{:?}", (size, width, real_width))), 
                    |bench| {
                        bench.iter(|| vc.prove());
                    }
                );
            }
        }
    }
    // verifier
    for &size in SIZES.iter(){
        for &width in WIDTHS.iter(){
            let max_pow = (width as f64).log2().ceil() as u32;
            for real_width in (1u32..=max_pow).map(|i| 2usize.pow(i)) 
            {
                let vc = vcminimal::VCMinimalExample::new(
                    options.clone(),
                    2,
                    size,
                    width,
                    16
                );
                group.bench_function(
                    BenchmarkId::from_parameter(format!("verifier{:?}", (size, width, real_width))), 
                    |bench| {
                        bench.iter_with_large_setup(
                            || {
                                let proof = vc.prove();
                                println!("\t Proof size: {:.1} KB", proof.to_bytes().len() as f64 / 1024f64);
                                proof
                            },
                            |proof| vc.verify(proof)
                        );
                    }
                );
            }
        }
    }
    group.finish();
}

criterion_group!(vcminimal_group, vcminimal);
criterion_main!(vcminimal_group);
