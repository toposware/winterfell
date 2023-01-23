// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2023 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::rescue::Rescue128;
use rand_utils::prng_vector;
use std::{cmp::Ordering, convert::TryInto};
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, StarkField},
    Serializable,
};

// CONSTANTS
// ================================================================================================

const MESSAGE_BITS: usize = 254;

// TYPES AND INTERFACES
// ================================================================================================

type KeyData = [BaseElement; 2];

pub struct PrivateKey {
    sec_keys: Vec<KeyData>,
    pub_keys: Vec<KeyData>,
    pub_key_hash: PublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey(KeyData);

pub struct Signature {
    pub ones: Vec<KeyData>,
    pub zeros: Vec<KeyData>,
}

// PRIVATE KEY IMPLEMENTATION
// ================================================================================================

impl PrivateKey {
    /// Returns a private key generated from the specified `seed`.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let keys_elements: Vec<BaseElement> = prng_vector(seed, MESSAGE_BITS * 2);
        let mut sec_keys = Vec::with_capacity(MESSAGE_BITS);
        let mut pub_keys = Vec::with_capacity(MESSAGE_BITS);

        for i in (0..keys_elements.len()).step_by(2) {
            let sk = [keys_elements[i], keys_elements[i + 1]];
            sec_keys.push(sk);

            let pk = Rescue128::digest(&sk).to_elements();
            pub_keys.push(pk);
        }

        let pub_key_hash = hash_pub_keys(&pub_keys);

        PrivateKey {
            sec_keys,
            pub_keys,
            pub_key_hash,
        }
    }

    /// Returns a public key corresponding to this private key.
    pub fn pub_key(&self) -> PublicKey {
        self.pub_key_hash
    }

    /// Signs the specified 'message` with this private key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        let mut ones = Vec::new();
        let mut zeros = Vec::new();

        let mut n = 0;
        let elements = message_to_elements(message);
        for element_bits in elements.iter().map(|e| e.to_repr()) {
            // make sure the most significant bit is 0
            assert_eq!(element_bits & (1 << 127), 0);
            for i in 0..127 {
                if (element_bits >> i) & 1 == 1 {
                    ones.push(self.sec_keys[n]);
                } else {
                    zeros.push(self.pub_keys[n]);
                }
                n += 1;
            }
        }

        Signature { ones, zeros }
    }
}

// PUBLIC KEY IMPLEMENTATION
// ================================================================================================

impl PublicKey {
    /// Returns true if the specified signature was generated by signing the specified message
    /// with a private key corresponding to this public key.
    pub fn verify(&self, message: &[u8], sig: &Signature) -> bool {
        let mut n_zeros = 0;
        let mut n_ones = 0;
        let mut pub_keys = Vec::with_capacity(MESSAGE_BITS);
        let elements = message_to_elements(message);
        for element_bits in elements.iter().map(|e| e.to_repr()) {
            // make sure the least significant bit is 0
            assert_eq!(element_bits & (1 << 127), 0);
            for i in 0..127 {
                if (element_bits >> i) & 1 == 1 {
                    if n_ones == sig.ones.len() {
                        return false;
                    }
                    pub_keys.push(Rescue128::digest(&sig.ones[n_ones]).to_elements());
                    n_ones += 1;
                } else {
                    if n_zeros == sig.zeros.len() {
                        return false;
                    }
                    pub_keys.push(sig.zeros[n_zeros]);
                    n_zeros += 1;
                }
            }
        }

        let pub_key_hash = hash_pub_keys(&pub_keys);
        *self == pub_key_hash
    }

    #[allow(dead_code, clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        bytes[..16].copy_from_slice(&self.0[0].to_bytes());
        bytes[16..].copy_from_slice(&self.0[1].to_bytes());
        bytes
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_elements(&self) -> [BaseElement; 2] {
        self.0
    }
}

impl Default for PublicKey {
    fn default() -> Self {
        PublicKey([BaseElement::ZERO; 2])
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0[0] == other.0[0] {
            self.0[1].to_repr().cmp(&other.0[1].to_repr())
        } else {
            self.0[0].to_repr().cmp(&other.0[0].to_repr())
        }
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// HELPER FUNCTIONS
// ================================================================================================

pub fn message_to_elements(message: &[u8]) -> [BaseElement; 2] {
    // reduce the message to a 32-byte value
    let hash = *blake3::hash(message).as_bytes();

    // interpret 32 bytes as two 128-bit integers
    let mut m0 = u128::from_le_bytes(hash[..16].try_into().unwrap());
    let mut m1 = u128::from_le_bytes(hash[16..].try_into().unwrap());

    // clear the most significant bit of the first value to ensure that it fits into 127 bits
    m0 = (m0 << 1) >> 1;

    // do the same thing with the second value, but also clear 8 more bits to make room for
    // checksum bits
    m1 = (m1 << 9) >> 9;

    // compute the checksum and put it into the most significant bits of the second values;
    // specifically: bit 127 is zeroed out, and 8 bits of checksum should go into bits
    // 119..127 thus, we just shift the checksum left by 119 bits and OR it with m1 (which
    // has top 9 bits zeroed out)
    let checksum = m0.count_zeros() + m1.count_zeros();
    let m1 = m1 | ((checksum as u128) << 119);

    [BaseElement::from(m0), BaseElement::from(m1)]
}

/// Reduces a list of public key elements to a single 32-byte value. The reduction is done
/// by breaking the list into two equal parts, and then updating hash state by taking turns
/// drawing elements from each list. For example, the final hash would be equivalent to:
/// hash(key[0] | key[127] | key[1] | key[128] | key[2] | key[129] ... )
/// This hashing methodology is implemented to simplify AIR design.
fn hash_pub_keys(keys: &[KeyData]) -> PublicKey {
    let mut pub_key_hash = Rescue128::new();
    pub_key_hash.update(&[BaseElement::ZERO; 4]);
    for i in 0..(MESSAGE_BITS / 2) {
        pub_key_hash.update(&keys[i]);
        pub_key_hash.update(&keys[i + MESSAGE_BITS / 2]);
    }

    PublicKey(pub_key_hash.finalize().to_elements())
}
