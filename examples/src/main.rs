// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use log::debug;
use std::io::Write;
use std::time::Instant;
use structopt::StructOpt;
use winterfell::StarkProof;

use examples::{cairo_cpu, pedersen_hash, fibonacci, rescue::*, vcminimal, vdf, ExampleOptions, ExampleType};
#[cfg(feature = "std")]
use examples::{lamport, merkle, rescue_raps};

// EXAMPLE RUNNER
// ================================================================================================

fn main() {
    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug)
        .init();

    // read command-line args
    let options = ExampleOptions::from_args();

    debug!("============================================================");

    // instantiate and prepare the example
    let example = match options.example {
        ExampleType::CairoCpu {num_steps} => {
            cairo_cpu::get_example(options, num_steps)
        }
        ExampleType::PedersenHash {num_steps} => {
            pedersen_hash::get_example(options, num_steps)
        }
        ExampleType::Fib { sequence_length } => {
            fibonacci::fib2::get_example(options, sequence_length)
        }
        ExampleType::Fib8 { sequence_length } => {
            fibonacci::fib8::get_example(options, sequence_length)
        }
        ExampleType::Mulfib { sequence_length } => {
            fibonacci::mulfib2::get_example(options, sequence_length)
        }
        ExampleType::Mulfib8 { sequence_length } => {
            fibonacci::mulfib8::get_example(options, sequence_length)
        }
        ExampleType::VCMinimal { num_steps, initial, width, real_width } => {
            vcminimal::get_example(options, initial, num_steps, width, real_width)
        }
        ExampleType::Vdf { num_steps } => vdf::regular::get_example(options, num_steps),
        ExampleType::VdfExempt { num_steps } => vdf::exempt::get_example(options, num_steps),
        ExampleType::RescueF62 { chain_length } => rescue_62::get_example(options, chain_length),
        ExampleType::RescueF63 { chain_length } => rescue_63::get_example(options, chain_length),
        ExampleType::RescueF128 { chain_length } => rescue_128::get_example(options, chain_length),
        #[cfg(feature = "std")]
        ExampleType::RescueRaps { chain_length } => rescue_raps::get_example(options, chain_length),
        #[cfg(feature = "std")]
        ExampleType::Merkle { tree_depth } => merkle::get_example(options, tree_depth),
        #[cfg(feature = "std")]
        ExampleType::LamportA { num_signatures } => {
            lamport::aggregate::get_example(options, num_signatures)
        }
        #[cfg(feature = "std")]
        ExampleType::LamportT { num_signers } => {
            lamport::threshold::get_example(options, num_signers)
        }
    };

    // generate proof
    let now = Instant::now();
    let proof = example.prove();
    debug!(
        "---------------------\nProof generated in {} ms",
        now.elapsed().as_millis()
    );

    let proof_bytes = proof.to_bytes();
    debug!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);
    debug!("Proof security: {} bits", proof.security_level(true));
    #[cfg(feature = "std")]
    debug!(
        "Proof hash: {}",
        hex::encode(blake3::hash(&proof_bytes).as_bytes())
    );

    // verify the proof
    debug!("---------------------");
    let parsed_proof = StarkProof::from_bytes(&proof_bytes).unwrap();
    assert_eq!(proof, parsed_proof);
    let now = Instant::now();
    match example.verify(proof) {
        Ok(_) => debug!(
            "Proof verified in {:.1} ms",
            now.elapsed().as_micros() as f64 / 1000f64
        ),
        Err(msg) => debug!("Failed to verify proof: {}", msg),
    }
    debug!("============================================================");
}
