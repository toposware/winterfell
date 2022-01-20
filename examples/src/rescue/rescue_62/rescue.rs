// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::utils::{are_equal, EvaluationResult};
use winterfell::math::{fields::f62::BaseElement, FieldElement};

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
const ALPHA: u32 = 3;
const INV_ALPHA: u64 = 3074416663688030891;

const MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(4611624995532045608),
    BaseElement::new(1080),
    BaseElement::new(4611624995532045947),
    BaseElement::new(40),
    BaseElement::new(4611624995532017177),
    BaseElement::new(42471),
    BaseElement::new(4611624995532031817),
    BaseElement::new(1210),
    BaseElement::new(4611624995531164247),
    BaseElement::new(1277640),
    BaseElement::new(4611624995531616908),
    BaseElement::new(33880),
    BaseElement::new(4611624995507347817),
    BaseElement::new(35708310),
    BaseElement::new(4611624995520110777),
    BaseElement::new(925771),
];

const INV_MDS: [BaseElement; STATE_WIDTH * STATE_WIDTH] = [
    BaseElement::new(3835753351312808841),
    BaseElement::new(505991969560173536),
    BaseElement::new(3204166179701427449),
    BaseElement::new(1677338490489682849),
    BaseElement::new(3912489244545528721),
    BaseElement::new(2992699836076075320),
    BaseElement::new(1194730044134442280),
    BaseElement::new(1123330866308046354),
    BaseElement::new(1961047666138455920),
    BaseElement::new(4252451032307404410),
    BaseElement::new(2998036551482399275),
    BaseElement::new(11714741135833070),
    BaseElement::new(683203703041784644),
    BaseElement::new(778093106242032509),
    BaseElement::new(4295326984864553447),
    BaseElement::new(3466626196915722075),
];

pub const ARK: [[BaseElement; STATE_WIDTH * 2]; CYCLE_LENGTH] = [
    [
        BaseElement::new(84059200413209450),
        BaseElement::new(373178937564870477),
        BaseElement::new(3634665217539531222),
        BaseElement::new(1818526052796649294),
        BaseElement::new(43002828310905347),
        BaseElement::new(1339785607435899452),
        BaseElement::new(3327414099846103536),
        BaseElement::new(3720267036147955407),
    ],
    [
        BaseElement::new(1212405412276462983),
        BaseElement::new(2466189979681890486),
        BaseElement::new(3281929273804089803),
        BaseElement::new(2765007764398338029),
        BaseElement::new(3860595181968282485),
        BaseElement::new(1700923066901328573),
        BaseElement::new(1822808759769232537),
        BaseElement::new(2626543261588181859),
    ],
    [
        BaseElement::new(1180785654043706125),
        BaseElement::new(3278507323242511379),
        BaseElement::new(2247861773607994080),
        BaseElement::new(888978770346910833),
        BaseElement::new(4065117358798607593),
        BaseElement::new(2535691992117626933),
        BaseElement::new(1892086820688304873),
        BaseElement::new(3667546902495623291),
    ],
    [
        BaseElement::new(3667562026480151801),
        BaseElement::new(1900600439264387015),
        BaseElement::new(3743472215158074923),
        BaseElement::new(374156173151790171),
        BaseElement::new(400784247678292935),
        BaseElement::new(485831602057389304),
        BaseElement::new(688571586707975441),
        BaseElement::new(2014042310608406449),
    ],
    [
        BaseElement::new(1901799904671064373),
        BaseElement::new(3778005880135162580),
        BaseElement::new(2391930266556619031),
        BaseElement::new(832601436562668997),
        BaseElement::new(4214057760921055958),
        BaseElement::new(658692901801137352),
        BaseElement::new(1954112702930448136),
        BaseElement::new(2998795451098641832),
    ],
    [
        BaseElement::new(4456530904183667625),
        BaseElement::new(342629764430205425),
        BaseElement::new(3492755002973900683),
        BaseElement::new(3814835056106218482),
        BaseElement::new(607170086553088030),
        BaseElement::new(795069255518443540),
        BaseElement::new(1919302892442085635),
        BaseElement::new(3556741158917451700),
    ],
    [
        BaseElement::new(3561926676429326404),
        BaseElement::new(2767297584682563727),
        BaseElement::new(4173772503566563981),
        BaseElement::new(3636870786946711035),
        BaseElement::new(3150131705229414069),
        BaseElement::new(4376594263245035840),
        BaseElement::new(453430431573410085),
        BaseElement::new(57461235190982874),
    ],
    [
        BaseElement::new(1010715261332251889),
        BaseElement::new(3814226295063661614),
        BaseElement::new(612783221392610123),
        BaseElement::new(274680007677058177),
        BaseElement::new(4590496723747560349),
        BaseElement::new(3589444804033441211),
        BaseElement::new(2810438166424592924),
        BaseElement::new(4344573364555470373),
    ],
    [
        BaseElement::new(892997045795553014),
        BaseElement::new(1808709039791092904),
        BaseElement::new(4542836651138703729),
        BaseElement::new(3019149084362551708),
        BaseElement::new(2904712339388229319),
        BaseElement::new(885603324699348123),
        BaseElement::new(2655024237486468326),
        BaseElement::new(589339913251683230),
    ],
    [
        BaseElement::new(1641967306908921355),
        BaseElement::new(2209618786454888003),
        BaseElement::new(3506691578385905661),
        BaseElement::new(21251929053485279),
        BaseElement::new(3442460353589681627),
        BaseElement::new(3720862489098581928),
        BaseElement::new(1150646531154045107),
        BaseElement::new(4575835837757565626),
    ],
    [
        BaseElement::new(2946269058019272865),
        BaseElement::new(4556767058423040792),
        BaseElement::new(3423759454234576830),
        BaseElement::new(4352253608578664076),
        BaseElement::new(731551570890522135),
        BaseElement::new(4109944482420570488),
        BaseElement::new(1785316767441539800),
        BaseElement::new(4202149893859497949),
    ],
    [
        BaseElement::new(4515940521830299618),
        BaseElement::new(509427395813016816),
        BaseElement::new(2703455222057663874),
        BaseElement::new(2358933959583288586),
        BaseElement::new(4587265030045200994),
        BaseElement::new(437929932013931358),
        BaseElement::new(157878995536006837),
        BaseElement::new(9188722667849804),
    ],
    [
        BaseElement::new(3528060750917340760),
        BaseElement::new(2120338204854229159),
        BaseElement::new(1850197494439346282),
        BaseElement::new(3455441796492337339),
        BaseElement::new(3914056536964108377),
        BaseElement::new(2271623193895877944),
        BaseElement::new(3680193581756190987),
        BaseElement::new(3123247226226873029),
    ],
    [
        BaseElement::new(2609694948293632651),
        BaseElement::new(426698706492066394),
        BaseElement::new(698555533963097770),
        BaseElement::new(242609722274523402),
        BaseElement::new(1706096316215143515),
        BaseElement::new(394685350925065643),
        BaseElement::new(2770607924709542204),
        BaseElement::new(1787028432509679680),
    ],
    [BaseElement::ZERO; 8],
    [BaseElement::ZERO; 8],
];
