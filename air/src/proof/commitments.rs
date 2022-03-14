// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crypto::Hasher;
use utils::{
    collections::Vec, ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
    SliceReader,
};

// COMMITMENTS
// ================================================================================================
/// Commitments made by the prover during commit phase of the protocol.
///
/// These commitments include:
/// * Commitments to the extended execution trace.
/// * Commitment to the evaluations of constraint composition polynomial over LDE domain.
/// * Commitments to the evaluations of polynomials at all FRI layers.
///
/// Internally, the commitments are stored as a sequence of bytes. Thus, to retrieve the
/// commitments, [parse()](Commitments::parse) function should be used.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Commitments(Vec<u8>);

impl Commitments {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new Commitments struct initialized with the provided commitments.
    pub fn new<H: Hasher>(
        trace_root: H::Digest,
        // TODO: we may want to handle more than 1 sequence of RAP columns
        aux_trace_root: Option<H::Digest>,
        constraint_root: H::Digest,
        fri_roots: Vec<H::Digest>,
    ) -> Self {
        let mut bytes = Vec::new();
        bytes.write(trace_root);
        bytes.write(aux_trace_root);
        bytes.write(constraint_root);
        bytes.write(fri_roots);
        Commitments(bytes)
    }

    // PUBLIC METHODS
    // --------------------------------------------------------------------------------------------

    /// Adds the specified commitment to the list of commitments.
    pub fn add<H: Hasher>(&mut self, commitment: &H::Digest) {
        commitment.write_into(&mut self.0);
    }

    /// Adds the auxiliary extended execution trace commitment to the list
    /// of commitments.
    pub fn add_aux_trace_root<H: Hasher>(&mut self, aux_commitment: &Option<H::Digest>) {
        aux_commitment.write_into(&mut self.0);
    }

    // PARSING
    // --------------------------------------------------------------------------------------------

    /// Parses the serialized commitments into distinct parts.
    ///
    /// The parts are (in the order in which they appear in the tuple):
    /// 1. Extended execution trace commitment.
    /// 2. Auxiliary extended execution trace commitment.
    /// 3. Constraint composition polynomial evaluation commitment.
    /// 4. FRI layer commitments.
    ///
    /// # Errors
    /// Returns an error if the bytes stored in self could not be parsed into the requested number
    /// of commitments, or if there are any unconsumed bytes remaining after the parsing completes.
    #[allow(clippy::type_complexity)]
    pub fn parse<H: Hasher>(
        self,
        num_fri_layers: usize,
    ) -> Result<(H::Digest, Option<H::Digest>, H::Digest, Vec<H::Digest>), DeserializationError>
    {
        // +1 for constraint root, +1 for FRI remainder commitment
        // The trace root and the optional auxiliary trace root are read before
        let num_commitments = num_fri_layers + 2;
        let mut reader = SliceReader::new(&self.0);

        let trace_commitment = H::Digest::read_from(&mut reader)?;

        // Try reading the auxiliary trace root, if any.
        let aux_trace_commitment = Option::<H::Digest>::read_from(&mut reader)?;

        let commitments = H::Digest::read_batch_from(&mut reader, num_commitments)?;

        // make sure we consumed all available commitment bytes
        if reader.has_more_bytes() {
            return Err(DeserializationError::UnconsumedBytes);
        }

        Ok((
            trace_commitment,
            aux_trace_commitment,
            commitments[0],
            commitments[1..].to_vec(),
        ))
    }
}

impl Serializable for Commitments {
    /// Serializes `self` and writes the resulting bytes into the `target`.
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        assert!(self.0.len() < u16::MAX as usize);
        target.write_u16(self.0.len() as u16);
        target.write_u8_slice(&self.0);
    }
}

impl Deserializable for Commitments {
    /// Reads commitments from the specified `source` and returns the result.
    ///
    /// # Errors
    /// Returns an error of a valid Commitments struct could not be read from the specified
    /// `source`.
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let num_bytes = source.read_u16()? as usize;
        let result = source.read_u8_vec(num_bytes)?;
        Ok(Commitments(result))
    }
}
