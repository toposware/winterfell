// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::utils::{are_equal, EvaluationResult};
use winterfell::math::{fields::f63::BaseElement, FieldElement};

/// The number of rounds is set to 7 to provide 128-bit security level.
/// computed using algorithm 7 from https://eprint.iacr.org/2020/1143.pdf
/// with 40% security margin.
const NUM_ROUNDS: usize = 7;

const STATE_WIDTH: usize = 14;
const CYCLE_LENGTH: usize = NUM_ROUNDS + 1;

// HASH FUNCTION
// ================================================================================================

/// Implementation of Rescue hash function with a 14 element state and 7 rounds. Accepts a
/// 7-element input, and returns a 7-element digest.
pub fn hash(value: [BaseElement; 7], result: &mut [BaseElement]) {
    let mut state = BaseElement::zeroed_vector(STATE_WIDTH);
    state[..7].copy_from_slice(&value);
    for i in 0..NUM_ROUNDS {
        apply_round(&mut state, i);
    }
    result.copy_from_slice(&state[..7]);
}

pub fn merge(value1: [BaseElement; 7], value2: [BaseElement; 7], result: &mut [BaseElement]) {
    // initialize the state by copying the digest elements
    let mut state = [BaseElement::ZERO; STATE_WIDTH];
    state[..7].copy_from_slice(&value1);
    state[7..14].copy_from_slice(&value2);
    for i in 0..NUM_ROUNDS {
        apply_round(&mut state, i);
    }
    result.copy_from_slice(&state[..7]);
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
        state[i] *= state[i].square();
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

// RESCUE CONSTANTS
// ================================================================================================

#[allow(dead_code)]
const ALPHA: u32 = 3;

const INV_ALPHA: u64 = 3146514939656186539;

const MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(0x13042324ac95f6fe),
    BaseElement::new(0xbe01d5ef588e5c3),
    BaseElement::new(0x1ef4a2c3efceb4cf),
    BaseElement::new(0x3638240da7314106),
    BaseElement::new(0x22e537b648a5ad8c),
    BaseElement::new(0x22f7cccb2226a7ca),
    BaseElement::new(0x326a7ee042a62b01),
    BaseElement::new(0x95f8bb25d91f5dc),
    BaseElement::new(0x174861c0af81b09b),
    BaseElement::new(0x2b6c5235f9dbb74c),
    BaseElement::new(0xb6cc271a540b037),
    BaseElement::new(0x3a6747b53b94a78),
    BaseElement::new(0x417ffeb31960d6ec),
    BaseElement::new(0x247dbc),
    BaseElement::new(0x405afc4ab368d06a),
    BaseElement::new(0xecb9a0524b44b7f),
    BaseElement::new(0x338f0a6c6ca981d5),
    BaseElement::new(0x21b54a5ae9742e5b),
    BaseElement::new(0x12e346b04e369efd),
    BaseElement::new(0xfef99ffd54b0cb1),
    BaseElement::new(0x1d29d9654335a319),
    BaseElement::new(0x22e05613d1cdebbd),
    BaseElement::new(0x7db192269ca5e0f),
    BaseElement::new(0xc103f4a3bf43754),
    BaseElement::new(0x1bb7d874b31d9d41),
    BaseElement::new(0x3c624af3138defbe),
    BaseElement::new(0x15b2850478d3de0d),
    BaseElement::new(0x3e6b401f8fb),
    BaseElement::new(0x238fb791afaec1e),
    BaseElement::new(0x2cbeb2d4ff33f1d4),
    BaseElement::new(0x388b2483c66f6b26),
    BaseElement::new(0xc2760d9d63cb0e5),
    BaseElement::new(0x3418d4d4b16f3e6d),
    BaseElement::new(0xbe4677c0fc16ed),
    BaseElement::new(0x294bfc3a50aed12b),
    BaseElement::new(0x3b2e11762294120e),
    BaseElement::new(0x3d4932933577b970),
    BaseElement::new(0x3825048a4e592379),
    BaseElement::new(0x3fe313b11664616b),
    BaseElement::new(0x5f572294c6925c2),
    BaseElement::new(0x22af8dc2d8b32404),
    BaseElement::new(0x210e589ca425455f),
    BaseElement::new(0x5f3f357ddd6d7fd),
    BaseElement::new(0x25e671a7e19acee8),
    BaseElement::new(0x3727fbcb0fd80a8f),
    BaseElement::new(0x3101aeace9f7eeb1),
    BaseElement::new(0x3e475a6b07b96950),
    BaseElement::new(0x359f3cb9c3ed5710),
    BaseElement::new(0xcb8e29e2e509979),
    BaseElement::new(0x3d63644c06958740),
    BaseElement::new(0x24a2965f3292aeca),
    BaseElement::new(0x23a5d7d5efab270c),
    BaseElement::new(0x3df05f68b7485fff),
    BaseElement::new(0x3fbe61aba5b28f69),
    BaseElement::new(0x1afdb1006ebdab4a),
    BaseElement::new(0x1a84322f583b0e44),
    BaseElement::new(0x2d57da2d35d729b3),
    BaseElement::new(0xba0213b19f13cb4),
    BaseElement::new(0x31d7357455579de0),
    BaseElement::new(0xd8bc70859d2afd1),
    BaseElement::new(0x94be97b51bdccad),
    BaseElement::new(0x253950f99814a7a1),
    BaseElement::new(0x1d74d520f83fe7fa),
    BaseElement::new(0x40f92ef26785f48a),
    BaseElement::new(0x227fad899eae4fde),
    BaseElement::new(0x367ac275c9fa5ceb),
    BaseElement::new(0x39b2178fbb0be45f),
    BaseElement::new(0x232fdc5bbe213a00),
    BaseElement::new(0x1b19537fc40a9c8e),
    BaseElement::new(0x35bc122811949369),
    BaseElement::new(0x2ac4f4826baa8e8f),
    BaseElement::new(0x20570a2fb84fa9b6),
    BaseElement::new(0x1ec2631061312246),
    BaseElement::new(0x1742684652cfe403),
    BaseElement::new(0x3f88a8ef8b6f4628),
    BaseElement::new(0x15ba701ab126998f),
    BaseElement::new(0x4b24d34f078fc97),
    BaseElement::new(0x3e28b1461086751e),
    BaseElement::new(0xdc0bc0a711e9408),
    BaseElement::new(0x1356d90793fd14ca),
    BaseElement::new(0x156e995527862213),
    BaseElement::new(0x2407dc3512482db5),
    BaseElement::new(0x3e7c616eb7be4487),
    BaseElement::new(0x1836b266f3c732ed),
    BaseElement::new(0x350f99f53da3cafa),
    BaseElement::new(0xc69c87af8c1a885),
    BaseElement::new(0x31590daa5ec7cabe),
    BaseElement::new(0x302619829c26a2bc),
    BaseElement::new(0x2f506f3c716fac87),
    BaseElement::new(0x383399b9bc142cf),
    BaseElement::new(0x13dfde5877dda335),
    BaseElement::new(0x2373a1456e119c30),
    BaseElement::new(0x22b218e931d10c16),
    BaseElement::new(0x1fb0a11c2a42363d),
    BaseElement::new(0x11e229b386b3db63),
    BaseElement::new(0x23b126c120684246),
    BaseElement::new(0xc2c515dad30aa2b),
    BaseElement::new(0x393df2152b2be62d),
    BaseElement::new(0x9a44c1d89503e95),
    BaseElement::new(0xf0cac1b2990e952),
    BaseElement::new(0x187d6b5d8bf3fb5b),
    BaseElement::new(0x1909d75405265213),
    BaseElement::new(0x17d353e199a40f98),
    BaseElement::new(0x2a94b224fc6cce34),
    BaseElement::new(0x10c4da4dec76391),
    BaseElement::new(0x1f7906ccdeb751d7),
    BaseElement::new(0x3c84495e094b820e),
    BaseElement::new(0xc16fbf7bc390e12),
    BaseElement::new(0x25daf7925be9afd7),
    BaseElement::new(0x171287b9fffa30b2),
    BaseElement::new(0x25f924d11806be93),
    BaseElement::new(0x3058812a2f05c842),
    BaseElement::new(0x3dc3f1eb4e09e1df),
    BaseElement::new(0x1d4e069dd5a7c71e),
    BaseElement::new(0xff71f684ccd686f),
    BaseElement::new(0x247b6989e4b479b0),
    BaseElement::new(0x48ed68c86389739),
    BaseElement::new(0x7f68ed25f30e2d7),
    BaseElement::new(0xdd6a8e80d9eeb7e),
    BaseElement::new(0x18866069fe87d510),
    BaseElement::new(0x235ab563b5bf0030),
    BaseElement::new(0x1d9540931893ddf3),
    BaseElement::new(0x18fadb35de0abeb1),
    BaseElement::new(0x907f67e2f2938f9),
    BaseElement::new(0x17ae1297417a4a9d),
    BaseElement::new(0xa7835919c3b19e2),
    BaseElement::new(0x4ce1fc3d4b442d4),
    BaseElement::new(0x271bbb9987e5d0a4),
    BaseElement::new(0x198dab1e767cda4d),
    BaseElement::new(0x1d154a8d6c74ae90),
    BaseElement::new(0x1ce07d726cf38431),
    BaseElement::new(0x12ace41008c3fd4f),
    BaseElement::new(0x3319aa93ef23c46b),
    BaseElement::new(0xda82c06b0c6ead9),
    BaseElement::new(0x154860c0b0e237ca),
    BaseElement::new(0x9c9be92b6e8a922),
    BaseElement::new(0x170e36966587c7a6),
    BaseElement::new(0x1547992363602a38),
    BaseElement::new(0x30248cadfdd88ebf),
    BaseElement::new(0x3a977b1e7c46d165),
    BaseElement::new(0x1380ecc20c2f2ea1),
    BaseElement::new(0x23eed3df57e66395),
    BaseElement::new(0x14be6dbfdbec5f75),
    BaseElement::new(0x9e7e9756d65e14f),
    BaseElement::new(0x1731c6503cd8631c),
    BaseElement::new(0xbf040658a66eeb9),
    BaseElement::new(0x1e8104996398fcec),
    BaseElement::new(0x178dbbdb0ef955ff),
    BaseElement::new(0x2f5e48caf914efce),
    BaseElement::new(0x2438bc7fa8a1eed5),
    BaseElement::new(0x12078cb9d5da84f9),
    BaseElement::new(0x321bfb788088c228),
    BaseElement::new(0x35b43fccd8806115),
    BaseElement::new(0xc4a53b5482c0174),
    BaseElement::new(0x105170952930db5b),
    BaseElement::new(0x14f8894131607da1),
    BaseElement::new(0x556ae468bda243f),
    BaseElement::new(0x23a8dfb64718c7b0),
    BaseElement::new(0x2213d3cc5cf6aa41),
    BaseElement::new(0x95b86b8c517e218),
    BaseElement::new(0x363cc753446eb291),
    BaseElement::new(0x3e900f20b47c7f6d),
    BaseElement::new(0x351a24528b99c0a2),
    BaseElement::new(0x266adef04df7aed7),
    BaseElement::new(0x326581d73130a96c),
    BaseElement::new(0x1cce888842391617),
    BaseElement::new(0x3499d635e0b69065),
    BaseElement::new(0x3e27635b89d03d66),
    BaseElement::new(0x3027cd9f2508bbb9),
    BaseElement::new(0x3d9ecdcfcba44430),
    BaseElement::new(0x305210ba2921cf87),
    BaseElement::new(0x387be08b1d2bf320),
    BaseElement::new(0x37f82c34cd7a85b9),
    BaseElement::new(0x326891e414fb74f),
    BaseElement::new(0xfe91d2deeab7f18),
    BaseElement::new(0x39ac96b9b2384ed2),
    BaseElement::new(0x2c97c1162d46df54),
    BaseElement::new(0x8477784d7356777),
    BaseElement::new(0x1208dfb2dddea9c4),
    BaseElement::new(0x38f6a6591845c08a),
    BaseElement::new(0x246e6a67b8e2aca2),
    BaseElement::new(0xc69e10265d3d4cc),
    BaseElement::new(0x78c599e195e8d84),
    BaseElement::new(0x2f723faaeb7fc7b8),
    BaseElement::new(0x3186707052cf8035),
    BaseElement::new(0x1e598c4b6bf915e4),
    BaseElement::new(0xc50d80de78ab27f),
    BaseElement::new(0x6ed4967ffd05be3),
    BaseElement::new(0x16ab047643724c3c),
    BaseElement::new(0x32d95e7fdd60b127),
    BaseElement::new(0x2e41df2454964bb9),
    BaseElement::new(0x9510bd7e54cbc8b),
    BaseElement::new(0x3e1caaf36c179cc2),
    BaseElement::new(0x25bee32c3ca21837),
    BaseElement::new(0x1e8c46c6758aa7b2),
    BaseElement::new(0x2ce425acdc03a3ff),
];

const INV_MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(0x34f07231b5a8683a),
    BaseElement::new(0x19bf73d0f1e2c40a),
    BaseElement::new(0x8fa7d660264cf3b),
    BaseElement::new(0x23a69668583fdb2),
    BaseElement::new(0x2a9ed0883aa0f75e),
    BaseElement::new(0x17426695466ac7dc),
    BaseElement::new(0x2cba12f80822038),
    BaseElement::new(0x15cd0e2290a952b6),
    BaseElement::new(0x3dbc6a59513fbfb7),
    BaseElement::new(0x1681a1682adf97d2),
    BaseElement::new(0x4123ac24f80e4fc4),
    BaseElement::new(0x21a839b73e40db0b),
    BaseElement::new(0x27bc3bafe79187b8),
    BaseElement::new(0x375b1f73a454ca9f),
    BaseElement::new(0x2363f94aab561974),
    BaseElement::new(0x165d6b9e319b8e33),
    BaseElement::new(0x1d36357c0c8e5939),
    BaseElement::new(0xeb17eeb1f3fdb3e),
    BaseElement::new(0x211f03c0c4a38099),
    BaseElement::new(0x2396ac9839f34820),
    BaseElement::new(0x17e643ab9b11504a),
    BaseElement::new(0xd758111052f431e),
    BaseElement::new(0x211d4b002479b216),
    BaseElement::new(0x255fa007707285e6),
    BaseElement::new(0x3e6ec1a4d11bceea),
    BaseElement::new(0x85cc10da32a3666),
    BaseElement::new(0x28371c19d43369f9),
    BaseElement::new(0x3c5e7c67b032083),
    BaseElement::new(0x3296728566f89c31),
    BaseElement::new(0x116910e5330fedcd),
    BaseElement::new(0x30f27772fa4b5d2e),
    BaseElement::new(0x19b3244b3929d4b8),
    BaseElement::new(0x2cf535e0c1dc4763),
    BaseElement::new(0x390b903d8c2a87ef),
    BaseElement::new(0x4148f2888025b398),
    BaseElement::new(0x54cb8618a7a4009),
    BaseElement::new(0x31e7183713a36015),
    BaseElement::new(0x3d519ad0ae3a0ad1),
    BaseElement::new(0x37c4ca681f25f997),
    BaseElement::new(0x31fbd5e75f8c7bd1),
    BaseElement::new(0x28941a558279d7),
    BaseElement::new(0x3922885d43c9270e),
    BaseElement::new(0x39628276d368f25c),
    BaseElement::new(0x1a785632ad6ee6c7),
    BaseElement::new(0x13d3497cd835f1d1),
    BaseElement::new(0x16ed00fede1c850c),
    BaseElement::new(0x3a6570019331e39a),
    BaseElement::new(0x5a5009a478d9887),
    BaseElement::new(0x3a9673dede5260cc),
    BaseElement::new(0x370185d7229cb0e2),
    BaseElement::new(0x1b957dec606dcae3),
    BaseElement::new(0x2988d1de8ad21771),
    BaseElement::new(0x3bd23b3369605769),
    BaseElement::new(0x37cde359f936106),
    BaseElement::new(0x1e9d3e5a25ce5e07),
    BaseElement::new(0x38b7cafad3252970),
    BaseElement::new(0x226efdad3f24b9bc),
    BaseElement::new(0x1455a0c21c891bbc),
    BaseElement::new(0x293cbe0258c6a7bd),
    BaseElement::new(0x3caf1c8e06e97699),
    BaseElement::new(0x2873191a8457f02b),
    BaseElement::new(0x1a610b3f70cd07fa),
    BaseElement::new(0x122ac692e423a7cf),
    BaseElement::new(0x2859d53ca4cd1d00),
    BaseElement::new(0x2d79f2f5ebb641a8),
    BaseElement::new(0x54e67796d909d84),
    BaseElement::new(0x248728b17f831dc7),
    BaseElement::new(0x237602737e442065),
    BaseElement::new(0x3c38c86241894ee0),
    BaseElement::new(0x3af878e02df4e30f),
    BaseElement::new(0x4022c31d64e496b1),
    BaseElement::new(0x327a64516d767e24),
    BaseElement::new(0x282f61da7a396e97),
    BaseElement::new(0x14793102a4cfe987),
    BaseElement::new(0x40d262f6fa66b90),
    BaseElement::new(0x2b5ba21efae8e744),
    BaseElement::new(0x3b842da613a9a35),
    BaseElement::new(0x35bcde5cac274317),
    BaseElement::new(0x3ebb76b23a4f6c98),
    BaseElement::new(0x1eca38a8b20cd89c),
    BaseElement::new(0x33293667ca457562),
    BaseElement::new(0x954d134a628529c),
    BaseElement::new(0x3c9b2b52b7a6cf60),
    BaseElement::new(0x1d3d79e482398664),
    BaseElement::new(0x382854da23e6148c),
    BaseElement::new(0x1c91c6edc955967f),
    BaseElement::new(0x26b418bad26a59e0),
    BaseElement::new(0x22c14dafe1d73f0f),
    BaseElement::new(0x15047f44e47afa63),
    BaseElement::new(0x2edf68d234c9d282),
    BaseElement::new(0x1032ee310bee80c2),
    BaseElement::new(0x38d5e8b920ef0181),
    BaseElement::new(0x13b99946a58b2f03),
    BaseElement::new(0x1585b8f84d4d535d),
    BaseElement::new(0x312e96d2a21e023e),
    BaseElement::new(0x2550e3e41f7d3c7e),
    BaseElement::new(0x303d5f070d68a144),
    BaseElement::new(0x30e792cf56840a87),
    BaseElement::new(0xd10302dabec4f61),
    BaseElement::new(0x2ac242eb47cb96a9),
    BaseElement::new(0x40f4cd1d8ce9c263),
    BaseElement::new(0x37c3ffe359dd5dd4),
    BaseElement::new(0x396b3d87bebb6615),
    BaseElement::new(0x1c852c80d0ad943c),
    BaseElement::new(0x18a92b123fc745ce),
    BaseElement::new(0x9b3880635f215db),
    BaseElement::new(0x4134051e9b19b19e),
    BaseElement::new(0x112e6d947b43f713),
    BaseElement::new(0xb7cc42683865900),
    BaseElement::new(0x94345d60c7853f),
    BaseElement::new(0x3e0fa0739b4e1453),
    BaseElement::new(0x524971a8a65088a),
    BaseElement::new(0x37f9b0b81ccc5e9c),
    BaseElement::new(0x1b884e897bc0f4e1),
    BaseElement::new(0xb9d660756cd258f),
    BaseElement::new(0x39cd7a6ef9c816b5),
    BaseElement::new(0x1ac97c64bfdf3517),
    BaseElement::new(0x2b019ba24ff156a4),
    BaseElement::new(0x1c3902d3569fa615),
    BaseElement::new(0x2ae2f9441240606f),
    BaseElement::new(0x21b6c426a8b604db),
    BaseElement::new(0x3c5ba759b8d8be0b),
    BaseElement::new(0x13a68ea3f1b7bdae),
    BaseElement::new(0x3a7a4c8fd2e8b6d5),
    BaseElement::new(0x1606c1bbb1ecc874),
    BaseElement::new(0x23f203b9c610de2c),
    BaseElement::new(0x25e7874d07bf8ab6),
    BaseElement::new(0x37bf452926cf8943),
    BaseElement::new(0xaa33b2c875cf3b5),
    BaseElement::new(0x3877d62150461848),
    BaseElement::new(0x258c74be83d23304),
    BaseElement::new(0x403ee2b4956cea02),
    BaseElement::new(0x7c3c8a0e62ebc98),
    BaseElement::new(0x34fddaa96be37c28),
    BaseElement::new(0x2feb9f90feb4f4b0),
    BaseElement::new(0x38b02538b229d87c),
    BaseElement::new(0x2f9ee964619f293e),
    BaseElement::new(0x347179e6cfa447d1),
    BaseElement::new(0x3551f12b0b72e5e8),
    BaseElement::new(0x8330e3ea0e7662b),
    BaseElement::new(0x3b3447a23d3ba03f),
    BaseElement::new(0x17a27f1122a1811),
    BaseElement::new(0x2daf583d8b75c6c4),
    BaseElement::new(0x33a3e797102c4918),
    BaseElement::new(0x273154db679a391a),
    BaseElement::new(0x264fd7ee8e0a7623),
    BaseElement::new(0x1baf5a0769ad56ae),
    BaseElement::new(0x3b5a2b1dda462e3c),
    BaseElement::new(0x18b0959edf0042c9),
    BaseElement::new(0x1fadb38c2473d2e6),
    BaseElement::new(0x17303af9d462d5),
    BaseElement::new(0x27f971d2ad88e5f),
    BaseElement::new(0x3eaf1e122f2a1c1),
    BaseElement::new(0x7939c44904c5b10),
    BaseElement::new(0x14371977af327749),
    BaseElement::new(0x336df7845c9ac29e),
    BaseElement::new(0x98df13786d38e7c),
    BaseElement::new(0x28fd286961cebb60),
    BaseElement::new(0x3cde050764645d90),
    BaseElement::new(0xd74662adf161d1e),
    BaseElement::new(0x1738d7775638c4d7),
    BaseElement::new(0x233c2c43069537b9),
    BaseElement::new(0x3aedbb82dccb1f56),
    BaseElement::new(0x3e6dd0af5934d66e),
    BaseElement::new(0x311cb441c0dd6308),
    BaseElement::new(0x1ab0340a95a700b5),
    BaseElement::new(0x79c788aad0b929c),
    BaseElement::new(0x4043796d31b818eb),
    BaseElement::new(0x39666cf57cfe2c71),
    BaseElement::new(0x3537e9e5a58c7dda),
    BaseElement::new(0x174374d7cca8f37f),
    BaseElement::new(0x45d4ee701129e16),
    BaseElement::new(0x3a05cb1ac4b1a01d),
    BaseElement::new(0x1ee719f7c8aa8b15),
    BaseElement::new(0x3bed858755e9de15),
    BaseElement::new(0x1adf689b8541aba3),
    BaseElement::new(0x2cd8871f3a369365),
    BaseElement::new(0x22a62bcdad70bf8),
    BaseElement::new(0x14d398760e1b1cd7),
    BaseElement::new(0x1b6a7211e7395924),
    BaseElement::new(0x87729e7077f291a),
    BaseElement::new(0x28cef3e59550d0cc),
    BaseElement::new(0x3de7127b3df8d7ec),
    BaseElement::new(0xd81f09142f00271),
    BaseElement::new(0x1308036f1453ec2a),
    BaseElement::new(0x403352b46c892d3c),
    BaseElement::new(0x24636b897d090792),
    BaseElement::new(0x3cd8ceb0c1c74ba5),
    BaseElement::new(0xd95413502eca397),
    BaseElement::new(0x10e6ba8cb9da13f1),
    BaseElement::new(0x1735cededaf68085),
    BaseElement::new(0xce32bb493c7e6d7),
    BaseElement::new(0x2dc996ab9816e8ee),
    BaseElement::new(0x2e9af50552c76684),
    BaseElement::new(0x13d38ac927a0ebfa),
    BaseElement::new(0x17d25fc681655ebe),
];

pub const ARK: [[BaseElement; STATE_WIDTH * 2]; CYCLE_LENGTH] = [
    [
        BaseElement::new(0x1f0a24c4ed42b7df),
        BaseElement::new(0x23966eb7b343720e),
        BaseElement::new(0x14bbfa44ff5b743f),
        BaseElement::new(0xe664c9986cb8a9e),
        BaseElement::new(0x4119c0c05c7ecd7e),
        BaseElement::new(0x32ce8901c4293486),
        BaseElement::new(0x3e68c5c98d4b4cb8),
        BaseElement::new(0x2a63cb703b3572a0),
        BaseElement::new(0x370da12ca562d56d),
        BaseElement::new(0x1da6d3d90c15b05d),
        BaseElement::new(0x2eaf791c2a38d572),
        BaseElement::new(0x2bb3461b78a1f224),
        BaseElement::new(0x397fea4351111fe6),
        BaseElement::new(0x1fe11370e8a410d8),
        BaseElement::new(0x287cc57b73b216c4),
        BaseElement::new(0x31d43141acff6960),
        BaseElement::new(0x24a060674a8713ea),
        BaseElement::new(0x41181e510c8dbc78),
        BaseElement::new(0x28eea3b98c6b9ee7),
        BaseElement::new(0x3ce13e44655b3186),
        BaseElement::new(0xd825b0db466b46d),
        BaseElement::new(0x4d55c6b88df6972),
        BaseElement::new(0x11847585b3e06d1e),
        BaseElement::new(0x2686f84c862f4896),
        BaseElement::new(0x3faec01f47b5a468),
        BaseElement::new(0x32010b89ce5a5c16),
        BaseElement::new(0x3a8a353735812e88),
        BaseElement::new(0x19acb2c8c419d69),
    ],
    [
        BaseElement::new(0x2a67f2def9434b18),
        BaseElement::new(0x5938a7b8a911856),
        BaseElement::new(0x345ae70ea4fdf960),
        BaseElement::new(0x383b69f66d65b559),
        BaseElement::new(0x1eea20fb14fcc9cf),
        BaseElement::new(0x40d6bd565c2cf37),
        BaseElement::new(0x368944bc3e1ae57d),
        BaseElement::new(0x3449b6bb664d184a),
        BaseElement::new(0x416c90ce460c7258),
        BaseElement::new(0x1e270c06d813795d),
        BaseElement::new(0xf9e18710b4874f2),
        BaseElement::new(0x2c13bdf5b184c1b2),
        BaseElement::new(0xce723b3c4e32ff6),
        BaseElement::new(0x3b4f9580c0a03588),
        BaseElement::new(0x309fa4dadff08e09),
        BaseElement::new(0xdb312001f5d8e61),
        BaseElement::new(0x3553e9ffa77bc9ee),
        BaseElement::new(0x177ddfc84dcab572),
        BaseElement::new(0x3a2e9ce68b1a5115),
        BaseElement::new(0x7858b9979f77e46),
        BaseElement::new(0x29f254b2d69334fc),
        BaseElement::new(0x2104a7ca8fb2d70f),
        BaseElement::new(0x394054b0791650e9),
        BaseElement::new(0x246e4e5b18f07e54),
        BaseElement::new(0x6fedf25cfedde0c),
        BaseElement::new(0x31c2caf0082ccb62),
        BaseElement::new(0x236548e19637b41a),
        BaseElement::new(0x29f6a61610faf9c1),
    ],
    [
        BaseElement::new(0x819707ec9a67813),
        BaseElement::new(0x20c2f6a293cc0a87),
        BaseElement::new(0x1d3709b68192a421),
        BaseElement::new(0x645fe7901df574e),
        BaseElement::new(0x21f889c67d7e3ba3),
        BaseElement::new(0x2c460161b3914236),
        BaseElement::new(0x1bac0ef8e49616b0),
        BaseElement::new(0x70aff1238d34e11),
        BaseElement::new(0x38c168bc2f68832b),
        BaseElement::new(0x412a168e21bf2b53),
        BaseElement::new(0x287ba21a54154ce6),
        BaseElement::new(0x21ff3f2653cdd1eb),
        BaseElement::new(0x2f173cca8668ff8f),
        BaseElement::new(0x3a8696d71835f516),
        BaseElement::new(0x6d60271f19bdec5),
        BaseElement::new(0xb3f07f7039df8bf),
        BaseElement::new(0x345c46a0cc5fb5ed),
        BaseElement::new(0x1f284a385da803d2),
        BaseElement::new(0x31272f6ad3863843),
        BaseElement::new(0x1ce856afd3537362),
        BaseElement::new(0x1b008de8c1c3ca3a),
        BaseElement::new(0x3acbde69cfc423a3),
        BaseElement::new(0x1fb8d8f1e44dfd37),
        BaseElement::new(0x25bcbadef12e3474),
        BaseElement::new(0x3ce171702963c13d),
        BaseElement::new(0x276fd7aed3f312a2),
        BaseElement::new(0x40a4e27e824c2d),
        BaseElement::new(0x3e9c674a8002ca62),
    ],
    [
        BaseElement::new(0x1239d2a50c98ee11),
        BaseElement::new(0x128aa086b005e82c),
        BaseElement::new(0x2a7e981d4efe8b00),
        BaseElement::new(0xaaec7ccfe7f2324),
        BaseElement::new(0x15e97e0d0e1b7358),
        BaseElement::new(0x447697cb53e2335),
        BaseElement::new(0x353490eeef4f707c),
        BaseElement::new(0xa844d78c57c82ae),
        BaseElement::new(0x37209c0bb193f4a),
        BaseElement::new(0x39ed12078e2206da),
        BaseElement::new(0x3f1f3091852b09cc),
        BaseElement::new(0x208c0a8b88fc9e3e),
        BaseElement::new(0x1444d6073161e6e3),
        BaseElement::new(0x1393abe3ac44a731),
        BaseElement::new(0x954901d34f08c2f),
        BaseElement::new(0x434d68beff8bc3c),
        BaseElement::new(0x289b878613113d7b),
        BaseElement::new(0x11571f4113f74aea),
        BaseElement::new(0x295d7a74aecbd738),
        BaseElement::new(0x3fc9cb8bc9e5ce6b),
        BaseElement::new(0xdbd33109a6a49f7),
        BaseElement::new(0x1322f7c31be4be9f),
        BaseElement::new(0x1ce0bb10c065e5d3),
        BaseElement::new(0xb952ab8628cb682),
        BaseElement::new(0x40f814133438cbdf),
        BaseElement::new(0x25722ec1766cd448),
        BaseElement::new(0x5d49fd46561472d),
        BaseElement::new(0x991bb35cb7052ca),
    ],
    [
        BaseElement::new(0x2823a1b0d2646a2c),
        BaseElement::new(0x3a0dc712d799b107),
        BaseElement::new(0x8c6e77050f662e4),
        BaseElement::new(0x2fbcf9c0fe368312),
        BaseElement::new(0x399d5795595c979b),
        BaseElement::new(0x3ddbaac6e5cab794),
        BaseElement::new(0x3e3abc3c104634a5),
        BaseElement::new(0x58218618c424b24),
        BaseElement::new(0x28d45bdc3c867372),
        BaseElement::new(0x1f04d7b485f02826),
        BaseElement::new(0x12b38b2b8757364d),
        BaseElement::new(0x8faf4eef692d005),
        BaseElement::new(0x1d9175f53e6c64a1),
        BaseElement::new(0x30cd988a0ce61ca3),
        BaseElement::new(0x1dd0bdfacfc9ff80),
        BaseElement::new(0x22245428977637c7),
        BaseElement::new(0x1ce88dba021e6543),
        BaseElement::new(0x40293474d9e4eb72),
        BaseElement::new(0xf6618f49fa7a229),
        BaseElement::new(0x12dad1e5ae9d67),
        BaseElement::new(0x36923eef059e5918),
        BaseElement::new(0xa5f8accc1bd2c6e),
        BaseElement::new(0xf7cb307d0f31bd5),
        BaseElement::new(0x26522ba1cc28828c),
        BaseElement::new(0x1090b6e701b628b6),
        BaseElement::new(0x3a97813f9a5a82eb),
        BaseElement::new(0x8fe5b3cb78acf),
        BaseElement::new(0x17d261078e8b32c3),
    ],
    [
        BaseElement::new(0x1e4acd6ff3382eef),
        BaseElement::new(0x3d17ca86a7651d49),
        BaseElement::new(0x2d804138338b7f72),
        BaseElement::new(0x152788e7fc018214),
        BaseElement::new(0x22bbf35179db337),
        BaseElement::new(0xeaae2acc8190a60),
        BaseElement::new(0x20196ecc727e035b),
        BaseElement::new(0x2698f7a3485ea605),
        BaseElement::new(0x19c8deaacaf65443),
        BaseElement::new(0xde9eb1d8981506a),
        BaseElement::new(0x3935397a2890ca7a),
        BaseElement::new(0xedf58cc48004974),
        BaseElement::new(0x136d5c0e55f1170b),
        BaseElement::new(0x2b2aa453d1bdb322),
        BaseElement::new(0x219c52e273d5a977),
        BaseElement::new(0x16dfe0dc6eb7456f),
        BaseElement::new(0x188f3d51ce9efd25),
        BaseElement::new(0x4605ce20f8e3da4),
        BaseElement::new(0x380547e70a777777),
        BaseElement::new(0x2b5c71584d414c99),
        BaseElement::new(0x37507fe409d8339d),
        BaseElement::new(0x2d40ddcb229e22ac),
        BaseElement::new(0x11ca5ec22bea5bf4),
        BaseElement::new(0xd7af7899ff5344b),
        BaseElement::new(0xb5a2470994c53da),
        BaseElement::new(0xa3c737f8f73b866),
        BaseElement::new(0x67bb69f329faed4),
        BaseElement::new(0xbace861a72434a),
    ],
    [
        BaseElement::new(0x3f000ab3f6cd3732),
        BaseElement::new(0xa30620299cb250),
        BaseElement::new(0xfe651ea8b4878a7),
        BaseElement::new(0x2c65270c6cc5551d),
        BaseElement::new(0x40a7e4c790eefa0c),
        BaseElement::new(0x1ed6c721331db81b),
        BaseElement::new(0x268cc27cc0f0dc74),
        BaseElement::new(0x34684cc51ff6b99f),
        BaseElement::new(0x24ded79a44672f43),
        BaseElement::new(0x1cdccb24cf696ed5),
        BaseElement::new(0x11ea5613d92dade1),
        BaseElement::new(0x1ab67d02d67b0c37),
        BaseElement::new(0xe09d70f3d80af1c),
        BaseElement::new(0x333b98b0e36e8bd8),
        BaseElement::new(0x3b4a7dec3394c686),
        BaseElement::new(0x19378054e5a32c32),
        BaseElement::new(0x40bc3d05a339f10),
        BaseElement::new(0x1f1d63e484ef0021),
        BaseElement::new(0x15ce5213567419f),
        BaseElement::new(0x1b08d33c33000502),
        BaseElement::new(0x3d6176e2dbb0bb17),
        BaseElement::new(0x1cdde9da8a6083c),
        BaseElement::new(0x2f552238068a6303),
        BaseElement::new(0x16794069461cc8dc),
        BaseElement::new(0x3ac5564dcac156b3),
        BaseElement::new(0x124db611cbae1828),
        BaseElement::new(0x128ca6911c1f9f69),
        BaseElement::new(0xba762a29f69c886),
    ],
    [BaseElement::ZERO; STATE_WIDTH * 2],
];
