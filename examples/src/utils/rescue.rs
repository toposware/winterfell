// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::utils::{are_equal, EvaluationResult};
use core::slice;
use winterfell::{
    crypto::{Digest, Hasher},
    math::{fields::f128::BaseElement, FieldElement},
    ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
};

/// Function state is set to 12 field elements or 96 bytes; 8 elements are reserved for rate
/// and 4 elements are reserved for capacity.
pub const STATE_WIDTH: usize = 12;
pub const RATE_WIDTH: usize = 8;

/// Two elements (32-bytes) are returned as digest.
const DIGEST_SIZE: usize = 2;

/// The number of rounds is set to 7 to provide 128-bit security level with 40% security margin;
/// computed using algorithm 7 from <https://eprint.iacr.org/2020/1143.pdf>
/// security margin here differs from Rescue Prime specification which suggests 50% security
/// margin (and would require 8 rounds) primarily to make AIR a bit simpler.
pub const NUM_ROUNDS: usize = 7;

/// Minimum cycle length required to describe Rescue permutation.
pub const CYCLE_LENGTH: usize = 8;

// TYPES AND INTERFACES
// ================================================================================================

pub struct Rescue128 {
    state: [BaseElement; STATE_WIDTH],
    idx: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Hash([BaseElement; DIGEST_SIZE]);

// RESCUE128 IMPLEMENTATION
// ================================================================================================

impl Rescue128 {
    /// Returns a new hasher with the state initialized to all zeros.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Rescue128 {
            state: [BaseElement::ZERO; STATE_WIDTH],
            idx: 0,
        }
    }

    /// Absorbs data into the hasher state.
    pub fn update(&mut self, data: &[BaseElement]) {
        for &element in data {
            self.state[self.idx] += element;
            self.idx += 1;
            if self.idx % RATE_WIDTH == 0 {
                apply_permutation(&mut self.state);
                self.idx = 0;
            }
        }
    }

    /// Returns hash of the data absorbed into the hasher.
    pub fn finalize(mut self) -> Hash {
        if self.idx > 0 {
            // TODO: apply proper padding
            apply_permutation(&mut self.state);
        }
        Hash([self.state[0], self.state[1]])
    }

    /// Returns hash of the provided data.
    pub fn digest(data: &[BaseElement]) -> Hash {
        // initialize state to all zeros
        let mut state = [BaseElement::ZERO; STATE_WIDTH];

        let mut i = 0;
        for &element in data.iter() {
            state[i] += element;
            i += 1;
            if i % RATE_WIDTH == 0 {
                apply_permutation(&mut state);
                i = 0;
            }
        }

        if i > 0 {
            // TODO: apply proper padding
            apply_permutation(&mut state);
        }

        Hash([state[0], state[1]])
    }
}

// HASHER IMPLEMENTATION
// ================================================================================================

impl Hasher for Rescue128 {
    type Digest = Hash;

    fn hash(_bytes: &[u8]) -> Self::Digest {
        unimplemented!("not implemented")
    }

    fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
        Self::digest(Hash::hashes_as_elements(values))
    }

    fn merge_with_int(_seed: Self::Digest, _value: u64) -> Self::Digest {
        unimplemented!("not implemented")
    }
}

// HASH IMPLEMENTATION
// ================================================================================================

impl Hash {
    pub fn new(v1: BaseElement, v2: BaseElement) -> Self {
        Hash([v1, v2])
    }

    #[allow(dead_code)]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        bytes[..16].copy_from_slice(&self.0[0].to_bytes());
        bytes[16..].copy_from_slice(&self.0[1].to_bytes());
        bytes
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_elements(&self) -> [BaseElement; DIGEST_SIZE] {
        self.0
    }

    pub fn hashes_as_elements(hashes: &[Hash]) -> &[BaseElement] {
        let p = hashes.as_ptr();
        let len = hashes.len() * DIGEST_SIZE;
        unsafe { slice::from_raw_parts(p as *const BaseElement, len) }
    }
}

impl Digest for Hash {
    fn as_bytes(&self) -> [u8; 32] {
        let bytes = BaseElement::elements_as_bytes(&self.0);
        let mut result = [0; 32];
        result[..bytes.len()].copy_from_slice(bytes);
        result
    }
}

impl Serializable for Hash {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.0[0]);
        target.write(self.0[1]);
    }
}

impl Deserializable for Hash {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let v1 = BaseElement::read_from(source)?;
        let v2 = BaseElement::read_from(source)?;
        Ok(Self([v1, v2]))
    }
}

// RESCUE PERMUTATION
// ================================================================================================

/// Applies Rescue-XLIX permutation to the provided state.
pub fn apply_permutation(state: &mut [BaseElement; STATE_WIDTH]) {
    // apply round function 7 times; this provides 128-bit security with 40% security margin
    for i in 0..NUM_ROUNDS {
        apply_round(state, i);
    }
}

/// Rescue-XLIX round function;
/// implementation based on algorithm 3 from <https://eprint.iacr.org/2020/1143.pdf>
#[inline(always)]
pub fn apply_round(state: &mut [BaseElement], step: usize) {
    // determine which round constants to use
    let ark = ARK[step % CYCLE_LENGTH];

    // apply first half of Rescue round
    apply_sbox(state);
    apply_mds(state);
    for i in 0..STATE_WIDTH {
        state[i] += ark[i];
    }

    // apply second half of Rescue round
    apply_inv_sbox(state);
    apply_mds(state);
    for i in 0..STATE_WIDTH {
        state[i] += ark[STATE_WIDTH + i];
    }
}

// CONSTRAINTS
// ================================================================================================

/// when flag = 1, enforces constraints for a single round of Rescue hash functions
pub fn enforce_round<E: FieldElement + From<BaseElement>>(
    result: &mut [E],
    current: &[E],
    next: &[E],
    ark: &[E],
    flag: E,
) {
    // compute the state that should result from applying the first half of Rescue round
    // to the current state of the computation
    let mut step1 = [E::ZERO; STATE_WIDTH];
    step1.copy_from_slice(current);
    apply_sbox(&mut step1);
    apply_mds(&mut step1);
    for i in 0..STATE_WIDTH {
        step1[i] += ark[i];
    }

    // compute the state that should result from applying the inverse for the second
    // half for Rescue round to the next step of the computation
    let mut step2 = [E::ZERO; STATE_WIDTH];
    step2.copy_from_slice(next);
    for i in 0..STATE_WIDTH {
        step2[i] -= ark[STATE_WIDTH + i];
    }
    apply_inv_mds(&mut step2);
    apply_sbox(&mut step2);

    // make sure that the results are equal
    for i in 0..STATE_WIDTH {
        result.agg_constraint(i, flag, are_equal(step2[i], step1[i]));
    }
}

// ROUND CONSTANTS
// ================================================================================================

/// Returns Rescue round constants arranged in column-major form.
pub fn get_round_constants() -> Vec<Vec<BaseElement>> {
    let mut constants = Vec::new();
    for _ in 0..(STATE_WIDTH * 2) {
        constants.push(vec![BaseElement::ZERO; CYCLE_LENGTH]);
    }

    #[allow(clippy::needless_range_loop)]
    for i in 0..CYCLE_LENGTH {
        for j in 0..(STATE_WIDTH * 2) {
            constants[j][i] = ARK[i][j];
        }
    }

    constants
}

// HELPER FUNCTIONS
// ================================================================================================

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_sbox<E: FieldElement>(state: &mut [E]) {
    for i in 0..STATE_WIDTH {
        state[i] = state[i].exp(ALPHA.into());
    }
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_inv_sbox(state: &mut [BaseElement]) {
    for i in 0..STATE_WIDTH {
        state[i] = state[i].exp(INV_ALPHA);
    }
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_mds<E: FieldElement + From<BaseElement>>(state: &mut [E]) {
    let mut result = [E::ZERO; STATE_WIDTH];
    let mut temp = [E::ZERO; STATE_WIDTH];
    for i in 0..STATE_WIDTH {
        for j in 0..STATE_WIDTH {
            temp[j] = E::from(MDS[i * STATE_WIDTH + j]) * state[j];
        }

        for j in 0..STATE_WIDTH {
            result[i] += temp[j];
        }
    }
    state.copy_from_slice(&result);
}

#[inline(always)]
#[allow(clippy::needless_range_loop)]
fn apply_inv_mds<E: FieldElement + From<BaseElement>>(state: &mut [E]) {
    let mut result = [E::ZERO; STATE_WIDTH];
    let mut temp = [E::ZERO; STATE_WIDTH];
    for i in 0..STATE_WIDTH {
        for j in 0..STATE_WIDTH {
            temp[j] = E::from(INV_MDS[i * STATE_WIDTH + j]) * state[j];
        }

        for j in 0..STATE_WIDTH {
            result[i] += temp[j];
        }
    }
    state.copy_from_slice(&result);
}

// CONSTANTS
// ================================================================================================

/// S-Box and Inverse S-Box powers;
/// computed using algorithm 6 from <https://eprint.iacr.org/2020/1143.pdf>
const ALPHA: u32 = 7;
const INV_ALPHA: u128 = 10540996611094048183;

/// Rescue MDS matrix
/// Computed using algorithm 4 from <https://eprint.iacr.org/2020/1143.pdf>
const MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
    BaseElement::new(23),
    BaseElement::new(23),
    BaseElement::new(8),
    BaseElement::new(26),
    BaseElement::new(13),
    BaseElement::new(10),
    BaseElement::new(9),
    BaseElement::new(7),
    BaseElement::new(6),
    BaseElement::new(22),
    BaseElement::new(21),
    BaseElement::new(8),
    BaseElement::new(7),
];

const INV_MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
    BaseElement::new(13278298489594233127),
    BaseElement::new(13278298489594233127),
    BaseElement::new(389999932707070822),
    BaseElement::new(9782021734907796003),
    BaseElement::new(4829905704463175582),
    BaseElement::new(7567822018949214430),
    BaseElement::new(14205019324568680367),
    BaseElement::new(15489674211196160593),
    BaseElement::new(17636013826542227504),
    BaseElement::new(16254215311946436093),
    BaseElement::new(3641486184877122796),
    BaseElement::new(11069068059762973582),
    BaseElement::new(14868391535953158196),
];

/// Rescue round constants;
/// computed using algorithm 5 from <https://eprint.iacr.org/2020/1143.pdf>
const ARK: [[BaseElement; STATE_WIDTH * 2]; NUM_ROUNDS] = [
    [
        BaseElement::new(16089809142501829443),
        BaseElement::new(3960375389654894755),
        BaseElement::new(2341987601489900096),
        BaseElement::new(16513505200733590422),
        BaseElement::new(2491992808872511534),
        BaseElement::new(2243959319871113313),
        BaseElement::new(1072250566756987431),
        BaseElement::new(9576211715023554739),
        BaseElement::new(13816740116943445245),
        BaseElement::new(1013981081016507493),
        BaseElement::new(6469202228346393176),
        BaseElement::new(651486455260752235),
        BaseElement::new(10659391161334081468),
        BaseElement::new(6658732499907968660),
        BaseElement::new(13472970356821082105),
        BaseElement::new(11254129182906430457),
        BaseElement::new(2200184099877207561),
        BaseElement::new(9367536782889046900),
        BaseElement::new(5776283441396365529),
        BaseElement::new(15880305242785227614),
        BaseElement::new(15064577366950298089),
        BaseElement::new(17182365414675952436),
        BaseElement::new(221227465681839092),
        BaseElement::new(10904420836212840752),
    ],
    [
        BaseElement::new(6770068611756627448),
        BaseElement::new(9429015895190610092),
        BaseElement::new(6345154718738704426),
        BaseElement::new(1348264131729825254),
        BaseElement::new(11257253180296854021),
        BaseElement::new(10209505772531486556),
        BaseElement::new(13936278878169192368),
        BaseElement::new(465229985152496221),
        BaseElement::new(16122840733837976660),
        BaseElement::new(15126432412337961371),
        BaseElement::new(18195743520412640434),
        BaseElement::new(4482481892207055145),
        BaseElement::new(9371429429698492981),
        BaseElement::new(15659859461375396037),
        BaseElement::new(3395558493871255061),
        BaseElement::new(660144660555450404),
        BaseElement::new(5074125520981119417),
        BaseElement::new(17453702653133595770),
        BaseElement::new(11221110160893954851),
        BaseElement::new(6495862879055376432),
        BaseElement::new(17061625752140729123),
        BaseElement::new(12368428993775985339),
        BaseElement::new(8908366829754037876),
        BaseElement::new(2078111330029178445),
    ],
    [
        BaseElement::new(4392703580426358869),
        BaseElement::new(1665895348145983),
        BaseElement::new(4219736658995217386),
        BaseElement::new(1227613135081507795),
        BaseElement::new(8190773212267744239),
        BaseElement::new(8282001820492621236),
        BaseElement::new(15836395107332526493),
        BaseElement::new(5607076305580595108),
        BaseElement::new(8785440730814333716),
        BaseElement::new(15628355668353690236),
        BaseElement::new(15635676168256493691),
        BaseElement::new(8231009457495604357),
        BaseElement::new(13168535446547922823),
        BaseElement::new(18239226123757899503),
        BaseElement::new(7641189915286036988),
        BaseElement::new(7820691679952216969),
        BaseElement::new(1111836394951152974),
        BaseElement::new(139835781513562161),
        BaseElement::new(7076109422888404220),
        BaseElement::new(5005587840202053100),
        BaseElement::new(6487413309175970078),
        BaseElement::new(5695661949695470409),
        BaseElement::new(18151333218502551049),
        BaseElement::new(12789465505850716019),
    ],
    [
        BaseElement::new(3242413417035426569),
        BaseElement::new(10974415453760425628),
        BaseElement::new(18279530845486603448),
        BaseElement::new(14045481066120861736),
        BaseElement::new(12525452082923300704),
        BaseElement::new(1905254592892409109),
        BaseElement::new(9346668368089967636),
        BaseElement::new(1735104742415647612),
        BaseElement::new(3317525224474295113),
        BaseElement::new(3946195652028520851),
        BaseElement::new(444992070656934445),
        BaseElement::new(3102693390775176900),
        BaseElement::new(17167036726114384788),
        BaseElement::new(5848569342998419381),
        BaseElement::new(14114543252495674018),
        BaseElement::new(15114629034072612072),
        BaseElement::new(5270549373288442547),
        BaseElement::new(12129247407828856056),
        BaseElement::new(18281855207204785420),
        BaseElement::new(597402865817114738),
        BaseElement::new(6042112508927673927),
        BaseElement::new(112810046686999112),
        BaseElement::new(2881728079621071110),
        BaseElement::new(3443512534203368354),
    ],
    [
        BaseElement::new(11524270175738513568),
        BaseElement::new(16596131169768068084),
        BaseElement::new(12046592239696686456),
        BaseElement::new(10335258789985873044),
        BaseElement::new(3804833210737803414),
        BaseElement::new(4871342344579357943),
        BaseElement::new(5506150606643613730),
        BaseElement::new(1144769156473837296),
        BaseElement::new(15770771149643607584),
        BaseElement::new(22835664835299105),
        BaseElement::new(15624512048862012204),
        BaseElement::new(8438597895149015250),
        BaseElement::new(13297012143576436426),
        BaseElement::new(7353183188832933627),
        BaseElement::new(14475065819552011569),
        BaseElement::new(1989958170371263671),
        BaseElement::new(2759712450935595252),
        BaseElement::new(5888211745553259072),
        BaseElement::new(3366223208861836535),
        BaseElement::new(10871170457430163614),
        BaseElement::new(7436939156294010029),
        BaseElement::new(10083282185253045512),
        BaseElement::new(1727628517966770716),
        BaseElement::new(15876537645083757620),
    ],
    [
        BaseElement::new(2077569020629574154),
        BaseElement::new(29247543278389127),
        BaseElement::new(7513950682870485886),
        BaseElement::new(14493142396838430095),
        BaseElement::new(13137935083971782251),
        BaseElement::new(17044896521696396448),
        BaseElement::new(8358879158995995396),
        BaseElement::new(6631372338926182917),
        BaseElement::new(16141080336903561376),
        BaseElement::new(12097878985033236818),
        BaseElement::new(16582826484887094232),
        BaseElement::new(11184522740344979309),
        BaseElement::new(14491184939776942308),
        BaseElement::new(16755331289686337123),
        BaseElement::new(4204064227783814013),
        BaseElement::new(17375825663893345502),
        BaseElement::new(16513382692712470059),
        BaseElement::new(12671191098792302109),
        BaseElement::new(7367953856881804491),
        BaseElement::new(4828831248603618923),
        BaseElement::new(605213678344474020),
        BaseElement::new(10779667723419446880),
        BaseElement::new(15588592678889744953),
        BaseElement::new(16719715619459928934),
    ],
    [
        BaseElement::new(11545814656420730331),
        BaseElement::new(7520668505762229291),
        BaseElement::new(5433441394427246897),
        BaseElement::new(17588828388580402390),
        BaseElement::new(8308794351872961990),
        BaseElement::new(14007549481740032380),
        BaseElement::new(15898890571959671932),
        BaseElement::new(812931430828255689),
        BaseElement::new(6818534534911166209),
        BaseElement::new(12562621953249472036),
        BaseElement::new(3817830678013523962),
        BaseElement::new(16954219307307160453),
        BaseElement::new(7976559292405617294),
        BaseElement::new(10624879739965265183),
        BaseElement::new(11858994588137577101),
        BaseElement::new(6953938202587799945),
        BaseElement::new(15487983798101099477),
        BaseElement::new(828942630404743552),
        BaseElement::new(15918441202173246890),
        BaseElement::new(10151280024237311966),
        BaseElement::new(10562603357011259664),
        BaseElement::new(18397974285238070711),
        BaseElement::new(878544804620014725),
        BaseElement::new(16579617335735550589),
    ],
];
