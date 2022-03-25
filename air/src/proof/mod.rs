// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Contains STARK proof struct and associated components.

use crate::{ProofOptions, TraceInfo, TraceLayout};
use core::cmp;
use fri::FriProof;
use math::log2;
use utils::{
    collections::Vec, ByteReader, Deserializable, DeserializationError, Serializable, SliceReader,
};

mod context;
pub use context::Context;

mod commitments;
pub use commitments::Commitments;

mod queries;
pub use queries::Queries;

mod ood_frame;
pub use ood_frame::OodFrame;

// CONSTANTS
// ================================================================================================

const GRINDING_CONTRIBUTION_FLOOR: u32 = 80;

// STARK PROOF
// ================================================================================================
/// A proof generated by Winterfell prover.
///
/// A STARK proof contains information proving that a computation was executed correctly. A proof
/// also contains basic metadata for the computation, but neither the definition of the computation
/// itself, nor public inputs consumed by the computation are contained in a proof.
///
/// A proof can be serialized into a sequence of bytes using [to_bytes()](StarkProof::to_bytes)
/// function, and deserialized from a sequence of bytes using [from_bytes()](StarkProof::from_bytes)
/// function.
///
/// To estimate soundness of a proof (in bits), [security_level()](StarkProof::security_level)
/// function can be used.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StarkProof {
    /// Basic metadata about the execution of the computation described by this proof.
    pub context: Context,
    /// Commitments made by the prover during the commit phase of the protocol.
    pub commitments: Commitments,
    /// Decommitments of extended execution trace values at positions queried by the verifier.
    pub trace_queries: Queries,
    /// Decommitments of constraint composition polynomial evaluations at positions queried by
    /// the verifier.
    pub constraint_queries: Queries,
    /// Trace and constraint polynomial evaluations at an out-of-domain point.
    pub ood_frame: OodFrame,
    /// Low-degree proof for a DEEP composition polynomial.
    pub fri_proof: FriProof,
    /// Proof-of-work nonce for query seed grinding.
    pub pow_nonce: u64,
}

impl StarkProof {
    /// Returns STARK protocol parameters used to generate this proof.
    pub fn options(&self) -> &ProofOptions {
        self.context.options()
    }

    /// Returns a layout describing how columns of the execution trace described by this context
    /// are arranged into segments.
    pub fn trace_layout(&self) -> &TraceLayout {
        self.context.trace_layout()
    }

    /// Returns trace length for the computation described by this proof.
    pub fn trace_length(&self) -> usize {
        self.context.trace_length()
    }

    /// Returns trace info for the computation described by this proof.
    pub fn get_trace_info(&self) -> TraceInfo {
        self.context.get_trace_info()
    }

    /// Returns the size of the LDE domain for the computation described by this proof.
    pub fn lde_domain_size(&self) -> usize {
        self.context.lde_domain_size()
    }

    // SECURITY LEVEL
    // --------------------------------------------------------------------------------------------
    /// Returns security level of this proof (in bits).
    ///
    /// When `conjectured` is true, conjectured security level is returned; otherwise, provable
    /// security level is returned. Usually, the number of queries needed for provable security is
    /// 2x - 3x higher than the number of queries needed for conjectured security at the same
    /// security level.
    pub fn security_level(&self, conjectured: bool) -> u32 {
        if conjectured {
            get_conjectured_security(
                self.context.options(),
                self.context.num_modulus_bits(),
                self.lde_domain_size() as u64,
            )
        } else {
            // TODO: implement provable security estimation
            unimplemented!("proven security estimation has not been implement yet")
        }
    }

    // SERIALIZATION / DESERIALIZATION
    // --------------------------------------------------------------------------------------------

    /// Serializes this proof into a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        self.context.write_into(&mut result);
        self.commitments.write_into(&mut result);
        self.trace_queries.write_into(&mut result);
        self.constraint_queries.write_into(&mut result);
        self.ood_frame.write_into(&mut result);
        self.fri_proof.write_into(&mut result);
        result.extend_from_slice(&self.pow_nonce.to_le_bytes());
        result
    }

    /// Returns a STARK proof read from the specified `source`.
    ///
    /// # Errors
    /// Returns an error of a valid STARK proof could not be read from the specified `source`.
    pub fn from_bytes(source: &[u8]) -> Result<Self, DeserializationError> {
        let mut source = SliceReader::new(source);
        let proof = StarkProof {
            context: Context::read_from(&mut source)?,
            commitments: Commitments::read_from(&mut source)?,
            trace_queries: Queries::read_from(&mut source)?,
            constraint_queries: Queries::read_from(&mut source)?,
            ood_frame: OodFrame::read_from(&mut source)?,
            fri_proof: FriProof::read_from(&mut source)?,
            pow_nonce: source.read_u64()?,
        };
        if source.has_more_bytes() {
            return Err(DeserializationError::UnconsumedBytes);
        }
        Ok(proof)
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Computes conjectured security level for the specified proof parameters.
fn get_conjectured_security(
    options: &ProofOptions,
    base_field_bits: u32,
    lde_domain_size: u64,
) -> u32 {
    // compute max security we can get for a given field size
    let field_size = base_field_bits * options.field_extension().degree();
    let field_security = field_size - lde_domain_size.trailing_zeros();

    // compute max security we can get for a given hash function
    let hash_fn_security = options.hash_fn().collision_resistance();

    // compute security we get by executing multiple query rounds
    let security_per_query = log2(options.blowup_factor());
    let mut query_security = security_per_query * options.num_queries() as u32;

    // include grinding factor contributions only for proofs adequate security
    if query_security >= GRINDING_CONTRIBUTION_FLOOR {
        query_security += options.grinding_factor();
    }

    cmp::min(
        cmp::min(field_security, query_security) - 1,
        hash_fn_security,
    )
}
