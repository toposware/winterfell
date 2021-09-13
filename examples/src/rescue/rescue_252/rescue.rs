// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::utils::{are_equal, EvaluationResult};
use winterfell::math::{fields::f252::BaseElement, FieldElement};

/// The number of rounds is set to 14 to provide 128-bit security level.
/// computed using algorithm 7 from https://eprint.iacr.org/2020/1143.pdf
const NUM_ROUNDS: usize = 14;

const STATE_WIDTH: usize = 4;
const CYCLE_LENGTH: usize = 16;

// HASH FUNCTION
// ================================================================================================

/// Implementation of Rescue hash function with a 4 element state and 14 rounds. Accepts a
/// 2-element input, and returns a 2-element digest.
pub fn hash(value: [BaseElement; 2], result: &mut [BaseElement]) {
    let mut state = BaseElement::zeroed_vector(STATE_WIDTH);
    state[..2].copy_from_slice(&value);
    for i in 0..NUM_ROUNDS {
        apply_round(&mut state, i);
    }
    result.copy_from_slice(&state[..2]);
}

// TRACE
// ================================================================================================

pub fn apply_round(state: &mut [BaseElement], step: usize) {
    // determine which round constants to use
    let ark = ARK[step % CYCLE_LENGTH];

    // apply first half of Rescue round
    apply_sbox(state);
    apply_mds(state);
    add_constants(state, &ark, 0);

    // apply second half of Rescue round
    apply_inv_sbox(state);
    apply_mds(state);
    add_constants(state, &ark, STATE_WIDTH);
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
fn add_constants(state: &mut [BaseElement], ark: &[BaseElement], offset: usize) {
    for i in 0..STATE_WIDTH {
        state[i] += ark[offset + i];
    }
}

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
        state[i] = state[i].exp(INV_ALPHA.into());
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

// RESCUE CONSTANTS
// ================================================================================================
const ALPHA: u32 = 3;
const INV_ALPHA: [u64; 4] = [
    0xaaaaaaaaaaaaaaab,
    0xaaaaaaaaaaaaaaaa,
    0xaaaaaaaaaaaaaaaa,
    0x0555555555555560,
];

const MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new([
        0xfffffffffffffd28,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x0000000000000438,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xfffffffffffffe7b,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x0000000000000028,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xffffffffffff8e19,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x000000000000a5e7,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xffffffffffffc749,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x00000000000004ba,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xfffffffffff28a57,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x0000000000137ec8,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xfffffffffff9728c,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x0000000000008458,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xfffffffffe872169,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x000000000220dd96,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
    BaseElement::new([
        0xffffffffff49e0b9,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x0800000000000010,
    ]),
    BaseElement::new([
        0x00000000000e204b,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ]),
];

const INV_MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new([
        0x0a2a5ec3c47f52cf,
        0x3d51f4232e9ecba1,
        0x8f3d16c2e09b45af,
        0x02a06782796baad9,
    ]),
    BaseElement::new([
        0x2938a37fba9ec402,
        0x03dd91892d34c334,
        0xb940a044ade7393b,
        0x009514dcbb7160db,
    ]),
    BaseElement::new([
        0x54844607a629f421,
        0x507367ab2c74c2fc,
        0x2560b149f8ebc561,
        0x0324ee73436072b7,
    ]),
    BaseElement::new([
        0x7818b7b4dab7f510,
        0x6e5d12a877b7ae2e,
        0x922197ae7891bbb4,
        0x01a5952d87c281a4,
    ]),
    BaseElement::new([
        0x019cddfd2a272607,
        0xb8f5de4315f0fdac,
        0xde57102aa9007b31,
        0x077a3158671cdc62,
    ]),
    BaseElement::new([
        0xb27161be7c912d71,
        0xd5f8aadc3d859fab,
        0x0cf4fecf876b26e0,
        0x052dbf9735fe8a64,
    ]),
    BaseElement::new([
        0x3390c5fa865f6df3,
        0xe21324e2cb616869,
        0x1a138c78ffe3445a,
        0x0653d17fe91fe586,
    ]),
    BaseElement::new([
        0x1860fa49d2e83e99,
        0x8efe51fde127fa3f,
        0xfaa0648ccfb11992,
        0x05043d9079c4b3e5,
    ]),
    BaseElement::new([
        0x93d743c668a5c019,
        0xcdc88409d5286253,
        0x4d41ab0490ae2da6,
        0x06ecaf953edbd483,
    ]),
    BaseElement::new([
        0xdabcc56eedef38d9,
        0xf9dfc550f698b7da,
        0x32ff4236dc245d7c,
        0x015deada1cf3a196,
    ]),
    BaseElement::new([
        0x8eb415472ec1d14f,
        0xfe87c2173a9e638c,
        0x3c9bd04b1f9e319a,
        0x00b7f57db4548a3c,
    ]),
    BaseElement::new([
        0x02b7e1837aa935c2,
        0x39cff48df9a08245,
        0x43234279738f4341,
        0x06fd7012efdbffcc,
    ]),
    BaseElement::new([
        0x425ed097b425ed0b,
        0x5ed097b425ed097b,
        0xd097b425ed097b42,
        0x004bda12f684bda1,
    ]),
    BaseElement::new([
        0x0b96a673e28086d9,
        0xb3183afef24df577,
        0x8a021b641511e8d2,
        0x041d7f7926fabb8e,
    ]),
    BaseElement::new([
        0xb69b3722102754a2,
        0xe7113506ac1242b8,
        0xeb47fd30cfe3e81e,
        0x03452e00b3cc070c,
    ]),
    BaseElement::new([
        0xfb6f51d25932377c,
        0x0705f8463bb2be54,
        0xba1e33452e00b3cc,
        0x005178732eb47fd3,
    ]),
];

pub const ARK: [[BaseElement; STATE_WIDTH * 2]; CYCLE_LENGTH] = [
    [
        BaseElement::new([
            0x5231b693310ca734,
            0x2c88e9d112801e64,
            0xa595640653cca094,
            0x01476d5a26457bb8,
        ]),
        BaseElement::new([
            0xd64a98401adacefa,
            0x5eb193ce1233c7e5,
            0x44e33938fffbdd13,
            0x004707d35dc8dc63,
        ]),
        BaseElement::new([
            0xa3e63e1f0d5bb084,
            0xa52322d70dd19ee2,
            0xa8347d664766b208,
            0x005e49fdaceff100,
        ]),
        BaseElement::new([
            0xdf4fd40f60f240af,
            0x7e86f0bd96b4db98,
            0x3dea1e0deb7ff4e0,
            0x05d33deb60cbf0eb,
        ]),
        BaseElement::new([
            0x250c859dc56be1ca,
            0xa0b75af0043bb8ae,
            0x5f43076fc2680b57,
            0x057368f5713920b1,
        ]),
        BaseElement::new([
            0xa21320bf7ec23efd,
            0x09a58ef5c3273d6e,
            0x2f2b19dffc56aef2,
            0x0202be22191b689d,
        ]),
        BaseElement::new([
            0x3c6e9c0c3c712bc3,
            0x9185a33bf84be6aa,
            0xa30b13a81374597f,
            0x06e8f70436939913,
        ]),
        BaseElement::new([
            0xdc9850d262d53b4f,
            0xe13d6a7c2d6b253c,
            0xff33dc4c1b5a7064,
            0x03f4a6d50c1fffca,
        ]),
    ],
    [
        BaseElement::new([
            0x27327752745e5dc7,
            0x5dfade217d48ad88,
            0x1393b47606f63413,
            0x00a65d39a5dba898,
        ]),
        BaseElement::new([
            0x2b8587339e88ed48,
            0xdb5d5ee3139c3625,
            0x4cb9ca6fc0d8926b,
            0x000b797b2aab807c,
        ]),
        BaseElement::new([
            0x81ccc8247d2f4a35,
            0x6271a19e34e76047,
            0xb6392c725fb4b906,
            0x049018bc062dfedc,
        ]),
        BaseElement::new([
            0xcc78063433caaa12,
            0xb991af1df2a062aa,
            0x23996d80602a629a,
            0x01ae78f27d3f90d4,
        ]),
        BaseElement::new([
            0xa9cf68e4cf72f452,
            0x3b37339240cd023e,
            0xfb95ae6ed9d0bcd8,
            0x05156bebf81ab533,
        ]),
        BaseElement::new([
            0xb4a571557e4d6a38,
            0xec5de377d15c49ec,
            0x47818b7bb3ead455,
            0x036ebb44fb57e172,
        ]),
        BaseElement::new([
            0x617a97b968e86906,
            0x24f3380be2050782,
            0xbf39ad2f9884d8cd,
            0x02415bd0f434f507,
        ]),
        BaseElement::new([
            0x0490a2bbf1739869,
            0x38e9c79bd7c20360,
            0xd41d99b0823a9eac,
            0x00c14b9a642c565e,
        ]),
    ],
    [
        BaseElement::new([
            0x37b4265f6f36f64d,
            0x5db64fa6ed54ceaa,
            0x54969a9cc6412668,
            0x034dfeda2f594482,
        ]),
        BaseElement::new([
            0xbc1041cb64f57df6,
            0x055d3fc4d393a979,
            0x27925b7c79a4fafd,
            0x069917b84dd12175,
        ]),
        BaseElement::new([
            0xbe43833c6289e7ac,
            0x857a1ea66f5fc6a7,
            0x55930222e38ea38a,
            0x02e5a78d010ca852,
        ]),
        BaseElement::new([
            0xc87338143246fe05,
            0xbf69074675f62e87,
            0x9272f7f29d024b38,
            0x030fb788000cb454,
        ]),
        BaseElement::new([
            0xb410910340011cd0,
            0xcb04f8ee31dce4da,
            0xe5c30babcc0af9c9,
            0x06702e8a1b5c2153,
        ]),
        BaseElement::new([
            0x09bf300f96d66bb1,
            0x7349f90c66310e6b,
            0x59c66e9c44f6ed40,
            0x076e7fa89c71ca07,
        ]),
        BaseElement::new([
            0xd5a414ade5342eef,
            0xdb14a7ad4531aca8,
            0x0177b2b580ed2882,
            0x0784c2928e52d3e6,
        ]),
        BaseElement::new([
            0x69d6f0a7c67ec384,
            0x59d20da103d81875,
            0x23ec7e6c63f3d36f,
            0x0222a8ed43c90e13,
        ]),
    ],
    [
        BaseElement::new([
            0xe2d92053a6564f52,
            0x1db4450581d7b6b2,
            0xdd800d88337641c6,
            0x01eaf08c6b55d381,
        ]),
        BaseElement::new([
            0xcfa9625edeb53410,
            0x9d921bd95d145f2e,
            0x46a07b0cba952f01,
            0x05c6211843e0d8a3,
        ]),
        BaseElement::new([
            0xf0e5d659b34f12ac,
            0x114212ce9bfd0b5b,
            0xa5dc7a5648124b51,
            0x0413607c6987c4af,
        ]),
        BaseElement::new([
            0x7e39f221abd52fdb,
            0x4df20085e4f34aed,
            0xa0692da2d3c3349f,
            0x029d1946fa184fed,
        ]),
        BaseElement::new([
            0xae7e2f2ead2471c8,
            0x0bfe50e87b9e68b7,
            0x474cd52d2cfbe074,
            0x07c641311a19adb9,
        ]),
        BaseElement::new([
            0x07fbb674b6c9225f,
            0xb69d96da7eeb4e98,
            0xc8691a2653fba5f7,
            0x06942c5b00dd7ef2,
        ]),
        BaseElement::new([
            0x3fd82b4064f3e992,
            0xe30977444e12c32c,
            0x9f93f4ab26e2c333,
            0x04b1fabb34e315c7,
        ]),
        BaseElement::new([
            0xfe012be1dab30046,
            0x1cbb32213b139c9f,
            0x28c15338f03164ff,
            0x03e91f9270f20712,
        ]),
    ],
    [
        BaseElement::new([
            0x0a1cf87035f4be4e,
            0x9b9c181d4e34b298,
            0xc4ca9ea71da77840,
            0x002aba71c3344681,
        ]),
        BaseElement::new([
            0xe4cb276f196ca2b2,
            0xe5df5d1925aa3250,
            0xbeba642d4c8f3e2a,
            0x00a436597fe2de92,
        ]),
        BaseElement::new([
            0x713db9478548f456,
            0xda879ec95475a743,
            0xb5e6b458c170575c,
            0x051fc01e694f3437,
        ]),
        BaseElement::new([
            0x82545c5f20832a13,
            0x867639ea078401fc,
            0x45d11693cb5a41a3,
            0x014f7d0f806a56e0,
        ]),
        BaseElement::new([
            0x84f26b7e0824f3aa,
            0x080a7f306b0abcde,
            0x236a7f592b93ed6c,
            0x01dde26fb4e1ea79,
        ]),
        BaseElement::new([
            0x50b77cb5972fd5d1,
            0x77babfe534d33773,
            0x973f391f34a9558b,
            0x031ab6aac2dcd105,
        ]),
        BaseElement::new([
            0xb9af0f83cdb4d3ae,
            0x28c298745be65f18,
            0x65ddb3413f3b1557,
            0x04c59f9909843cb8,
        ]),
        BaseElement::new([
            0xd2565f313672ceee,
            0x3f501236d472d6fc,
            0x75ae49174c157022,
            0x01aea710a140e9f4,
        ]),
    ],
    [
        BaseElement::new([
            0x480f306960b5c4fd,
            0x01633f73f8135452,
            0x4119bffcea9b352d,
            0x07c9e9b9b25839c5,
        ]),
        BaseElement::new([
            0x9f8c3ce88fcc4a54,
            0x69b1089ab786ea79,
            0xf54f80ff1d674152,
            0x0670f4d4b39bb303,
        ]),
        BaseElement::new([
            0x47eadd0e041aab34,
            0xfda789043d0ebba6,
            0x738053b034f34e31,
            0x0355f1dc68b387e6,
        ]),
        BaseElement::new([
            0x5fa5fe5858573156,
            0x62d0ffe0a7163213,
            0x1aef5efbaef47cbd,
            0x002ef429d182a147,
        ]),
        BaseElement::new([
            0x515cd69881974653,
            0xb52161e67972c512,
            0x0fb5b5717d7d7633,
            0x00f5d71406ddc9de,
        ]),
        BaseElement::new([
            0x0e8319885868893b,
            0x08bd481bc7bf2e75,
            0x04205ac3aa70c1a6,
            0x062e591b62fdb99f,
        ]),
        BaseElement::new([
            0xe63d8d7e86222289,
            0x4fc016957fdd6bc3,
            0x4e5cdf89510e26d8,
            0x0227a6861342bb9b,
        ]),
        BaseElement::new([
            0x74ea4b5c36276922,
            0xf9ef2aac5b5c031c,
            0x146a6a0822c5c026,
            0x067a287e24aa2ab1,
        ]),
    ],
    [
        BaseElement::new([
            0x7f3c952cb4217491,
            0xd0e1c2bc176658b0,
            0x1df2e8b11739e5e3,
            0x007b3fadf31424ba,
        ]),
        BaseElement::new([
            0xc583eb7518303a17,
            0x6610436fd3668ccc,
            0x7c5e1ee42165be0c,
            0x0701fd3e191feb5f,
        ]),
        BaseElement::new([
            0x75a0653604725505,
            0x294056398f2b3d77,
            0x78c0db28fafd6c38,
            0x0635bc275412546e,
        ]),
        BaseElement::new([
            0x87128d33f4ccf569,
            0x9bbc2ad2843163fd,
            0xd0083a67f1350bda,
            0x074517132055d54b,
        ]),
        BaseElement::new([
            0xf7e3b80d141d4c16,
            0xb1b85eba5c43c4e0,
            0x2da7aefc5008fdda,
            0x04e48bb9eb616cd5,
        ]),
        BaseElement::new([
            0x8da473a264407739,
            0x2142f0320c3203bd,
            0x154a5528f18bf1b5,
            0x006accf110232575,
        ]),
        BaseElement::new([
            0xc4b20856571278f2,
            0x6b684a1606c7b561,
            0x969e79073dd82255,
            0x060e7d9d87cd0807,
        ]),
        BaseElement::new([
            0x387c740b4021a165,
            0x109df0fc299b52fe,
            0xc6a685e2ff6c41f1,
            0x0539a768cddbb267,
        ]),
    ],
    [
        BaseElement::new([
            0x6cc51c5f632b9b13,
            0x0927ceac9a7c3739,
            0xc63780633be680a0,
            0x038ef96e35cf9fb7,
        ]),
        BaseElement::new([
            0xdd14b91335729c70,
            0xd88c6c37a6c3b967,
            0x8c484dd54e368672,
            0x02ecf120ea6f81d4,
        ]),
        BaseElement::new([
            0x7031df0aaefdefd7,
            0xe917ee989afbbf12,
            0x8f9025840688eee2,
            0x046cd5d38d0a8396,
        ]),
        BaseElement::new([
            0xcb5b0d28248927d8,
            0x3186c58de7a52e36,
            0x4bb5cbc8e6627a4e,
            0x0486e37d680ef7fa,
        ]),
        BaseElement::new([
            0x4685cba0b7bc78e5,
            0xe66b058660f83de8,
            0x68c8b03590edafd7,
            0x05ac126cf3d30129,
        ]),
        BaseElement::new([
            0x24a33f503000c7d3,
            0xe13500a24f39a0e7,
            0x32424c0f9329a1ba,
            0x014652de1ef95ea9,
        ]),
        BaseElement::new([
            0xe9fcddfea730cf20,
            0xd7de724210b43c36,
            0x1fa2e74f2115b1a8,
            0x012c40228256de2f,
        ]),
        BaseElement::new([
            0x6ddaad428360ba47,
            0xef91829e62b34160,
            0xfd80356fa348d04f,
            0x0758dbd783add272,
        ]),
    ],
    [
        BaseElement::new([
            0x0369c764cd92d97f,
            0x12684019a3bc7f95,
            0x520fe79ab3eacd20,
            0x010c7152113606df,
        ]),
        BaseElement::new([
            0x892e02c1df426979,
            0x18c9d4caeb9d7bfa,
            0x0d952ce69fa0f72f,
            0x00306c074cee9d3c,
        ]),
        BaseElement::new([
            0xc3e14a325b421940,
            0xff3622c975408eb2,
            0xd8bc470db54445cf,
            0x07ed19e7f62ffd07,
        ]),
        BaseElement::new([
            0x115370598cf9f095,
            0x1844e441a7405bab,
            0xb7a0a0bf297a7389,
            0x000cf1dd0dab0ca3,
        ]),
        BaseElement::new([
            0x7fc39996fb689289,
            0x3cd8a0494b255af0,
            0x181eeaeeec71f3e4,
            0x04523fa5a806ba72,
        ]),
        BaseElement::new([
            0xce7b664a52067ef1,
            0xb5fa1f60a0c6aef6,
            0x2f23e5490185ca41,
            0x07925670a375fca5,
        ]),
        BaseElement::new([
            0x5fdb3a02d4f2a3f9,
            0xd3f3a58662c07e4e,
            0x15b966cee41c5257,
            0x04b3f88ed79b1666,
        ]),
        BaseElement::new([
            0x889b2a258e897d96,
            0x07df5c75cd71ad41,
            0xe3c1ed5a62ffca1a,
            0x023c4ceccdcb77d8,
        ]),
    ],
    [
        BaseElement::new([
            0x724dec4a7955566c,
            0x3ccdbca65d9e4dae,
            0x3f2d2ccf42424d81,
            0x02bf719dd2bebecb,
        ]),
        BaseElement::new([
            0x94acb5a6de52a22b,
            0x37213c8156208b59,
            0x4e143deaeb4c8c3d,
            0x013829f9cf45893d,
        ]),
        BaseElement::new([
            0x1680ca069d6c3acd,
            0x1dba87d8d638de26,
            0x8ba819efbe18c503,
            0x0799a41302507647,
        ]),
        BaseElement::new([
            0x9b6c3647c96f6f02,
            0x283eea3c0e4117b4,
            0x397a9a2ca167ecd0,
            0x0210166cdd2d6c03,
        ]),
        BaseElement::new([
            0xf38588d1082135bb,
            0x05bebf117c1ea321,
            0x96c36c63ef3c774c,
            0x03fa910715f489ae,
        ]),
        BaseElement::new([
            0x23c91a4123537e25,
            0x24e927ec2ceec76e,
            0xf5a405e062f438ef,
            0x032585fb16ffa31b,
        ]),
        BaseElement::new([
            0xb618a3f07a036fe9,
            0xf3041499bc40c228,
            0x55caeca9cb4cf491,
            0x0129eeb480b893bb,
        ]),
        BaseElement::new([
            0x3c00c89a0b5ce050,
            0x2345be301ad1de71,
            0x63cc1a310cdef0a4,
            0x079e377d84267734,
        ]),
    ],
    [
        BaseElement::new([
            0xc46ba60205a9b616,
            0x82b4644bdfe2049e,
            0xdb073e408d6a074d,
            0x0366b4d09da69420,
        ]),
        BaseElement::new([
            0x1b20673e5f94d389,
            0x58836e88f8756c20,
            0x868c4a451f7fb367,
            0x03087c44ef43d657,
        ]),
        BaseElement::new([
            0x856c7bbf1ef918f5,
            0xe7357d01500efb19,
            0xdf15058a69b8533a,
            0x031933bafc6caa01,
        ]),
        BaseElement::new([
            0xca0bc007ba5090de,
            0x768b50958fcb340f,
            0xc185f692a2565a39,
            0x012d5d44b3fdd9e4,
        ]),
        BaseElement::new([
            0x862c477eb4ef477f,
            0xbd0164de0d9cfa63,
            0x86ac469d8325002c,
            0x00ce7a9de5d6d548,
        ]),
        BaseElement::new([
            0x6312f815949380a9,
            0x38964c1d58ba982a,
            0x568ab2322ec0b554,
            0x0250f3be14075977,
        ]),
        BaseElement::new([
            0x9a3c9fc88e038d30,
            0xa172ee10056edb11,
            0xa2c66777f786441c,
            0x04694f04f3059e85,
        ]),
        BaseElement::new([
            0x31823fcea8cfdf7e,
            0x67a3d4f0ec086a6e,
            0xbf825593f3acfba4,
            0x04962923899e997c,
        ]),
    ],
    [
        BaseElement::new([
            0x43410e74af23c6bc,
            0x591e07cb92d46e37,
            0xe223d82264bf9122,
            0x02d14ee6b0b3babe,
        ]),
        BaseElement::new([
            0xfa55db2399d6cd8e,
            0xc324d5f9b2c7b37f,
            0x0b39b18b0bdce56b,
            0x01b735dd2387699c,
        ]),
        BaseElement::new([
            0x4b4dc197f85e4568,
            0xa13d1b1d7f1eba31,
            0x5fd56951c39e2f8e,
            0x077920655281828f,
        ]),
        BaseElement::new([
            0xcab1ff5c5c653060,
            0xe1bb073582d70c8d,
            0x47fa526ecb6192cb,
            0x012edc896922ab32,
        ]),
        BaseElement::new([
            0x125bfce53d0dc1af,
            0x8af16d66d1e1e0aa,
            0x200e6c4efa34636c,
            0x065445d63f859765,
        ]),
        BaseElement::new([
            0xea476c4e72aaf7f8,
            0xf72405e4b43be7a7,
            0x1e5ec7a19789c540,
            0x048d756370878a7a,
        ]),
        BaseElement::new([
            0xf3c084c60819e64f,
            0xa827d9d0a5ec528e,
            0x5d72b3985c41b9ce,
            0x036630b55b535917,
        ]),
        BaseElement::new([
            0x9806c64b5c2a0279,
            0x083596f6ab78b4f3,
            0x642a0c46816bf28a,
            0x0173cf4fb3e1e449,
        ]),
    ],
    [
        BaseElement::new([
            0xd8eba27511733ebf,
            0xbafdd8c9e1324ff9,
            0x68b452d0eea28fda,
            0x056a06a6bce6eb63,
        ]),
        BaseElement::new([
            0xa05279c0b3ba22b4,
            0xc8d954054dd23f9b,
            0xf0b0d781049ef55e,
            0x01f9c570a584be87,
        ]),
        BaseElement::new([
            0xea7cc6c2e225eea3,
            0x866b045c223958f1,
            0xbfe8d5a1ab8e7fe6,
            0x07ff85df9134ba4a,
        ]),
        BaseElement::new([
            0x0a21e57f3bb8c220,
            0x8ebb98d113de2315,
            0xef055b8c8a3954bf,
            0x03fee11db2927d0f,
        ]),
        BaseElement::new([
            0x3bfbf09f84ca85ab,
            0x949b9eaa320a7aa9,
            0xeb116d969583b6e3,
            0x050ffaf4e49bf417,
        ]),
        BaseElement::new([
            0xbaf0abb7bb6ad183,
            0x3f73d1836d25dee4,
            0x9031b2bda8d65754,
            0x05983219d02fe395,
        ]),
        BaseElement::new([
            0x557c5629aafb3d08,
            0xefdde43201675ffc,
            0x3ea72bd04019a9cd,
            0x028900839c299607,
        ]),
        BaseElement::new([
            0xeee102be02487855,
            0xd02ff2cbf4821d8c,
            0xa8e957f404840d0a,
            0x03caf802861e4e23,
        ]),
    ],
    [
        BaseElement::new([
            0x6c00985f083d89af,
            0xaebad07648386063,
            0xf7ace104c3645b7e,
            0x04fb02246790f069,
        ]),
        BaseElement::new([
            0x00b1076cd472597c,
            0x594d3292cd15e686,
            0x4c0d02543a140264,
            0x01e3c4c53cee35d4,
        ]),
        BaseElement::new([
            0xe95b6df8bf4d8718,
            0x3a98e09524047322,
            0x534af4bb0d085598,
            0x02b401051a8d48d8,
        ]),
        BaseElement::new([
            0x3dc3a66f45919258,
            0xbce9cd4dba5324ae,
            0x2f71add9d70537d1,
            0x069bcdb9ccf7a8ae,
        ]),
        BaseElement::new([
            0x5d5dba02bb4a0705,
            0xe5bc8cc5f8625934,
            0x562d9454b8e6e0c4,
            0x02c9672e0aa829c3,
        ]),
        BaseElement::new([
            0x51626563c656f72d,
            0xb09774b042f9c19b,
            0xef4e43ba9bb88b7f,
            0x0341c3f54ef9dc96,
        ]),
        BaseElement::new([
            0xa6feb343bc021018,
            0xeba5bd1172b7e5c5,
            0x0124a60929c3baef,
            0x07f1d315d19d9edc,
        ]),
        BaseElement::new([
            0xc89e93bca621f394,
            0x840f1b61f5455f8c,
            0x4e9c11783a798734,
            0x031b030036d78d2f,
        ]),
    ],
    [BaseElement::ZERO; 8],
    [BaseElement::ZERO; 8],
];
