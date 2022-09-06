// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f64::BaseElement, log2, ExtensionOf, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

mod air;
use air::{CairoAir, PublicInputs};

mod prover;
use prover::CairoProver;

#[cfg(test)]
mod tests;

mod custom_trace_table;
pub use custom_trace_table::RapTraceTable;

use crate::utils::print_trace;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Mutex;

// CONSTANTS
// ================================================================================================

const TRACE_WIDTH: usize = 218;
const AUX_WIDTH: usize = 33;
const NB_OFFSET_COLUMNS: usize = 4;
const NB_MEMORY_COLUMN_PAIRS: usize = 29;

const OFFSET_COLUMNS: [usize; NB_OFFSET_COLUMNS] = [16, 17, 18, 33];

const SORTED_OFFSET_COLUMNS: [usize; NB_OFFSET_COLUMNS] = [34, 35, 36, 37];

const MEMORY_COLUMNS: [(usize, usize); NB_MEMORY_COLUMN_PAIRS] = [
    (19, 20),
    (21, 22),
    (23, 24),
    (25, 26),
    (38, 39),
    (50, 51),
    (52, 53),
    (54, 55),
    (56, 57),
    (58, 59),
    (60, 61),
    (62, 63),
    (64, 65),
    (66, 67),
    (68, 69),
    (70, 71),
    (72, 73),
    (74, 75),
    (76, 77),
    (78, 79),
    (80, 81),
    (82, 83),
    (84, 85),
    (86, 87),
    (88, 89),
    (90, 91),
    (92, 93),
    (94, 95),
    (96, 97),
];

const SORTED_MEMORY_COLUMNS: [(usize, usize); NB_MEMORY_COLUMN_PAIRS] = [
    (40, 41),
    (42, 43),
    (44, 45),
    (46, 47),
    (48, 49),
    (170, 171),
    (172, 173),
    (174, 175),
    (176, 177),
    (178, 179),
    (180, 181),
    (182, 183),
    (184, 185),
    (186, 187),
    (188, 189),
    (190, 191),
    (192, 193),
    (194, 195),
    (196, 197),
    (198, 199),
    (200, 201),
    (202, 203),
    (204, 205),
    (206, 207),
    (208, 209),
    (210, 211),
    (212, 213),
    (214, 215),
    (216, 217),
];

const STATE_WIDTH: usize = 12;

const NUM_ROUNDS: usize = 7;

/// S-Box and Inverse S-Box powers;
/// computed using algorithm 6 from <https://eprint.iacr.org/2020/1143.pdf>
const ALPHA: u32 = 7;
const INV_ALPHA: u64 = 10540996611094048183;

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

// CAIRO EXAMPLE
// ================================================================================================

pub fn get_example(
    options: ExampleOptions,
    trace_file_path: String,
    public_input_file_path: String,
) -> Box<dyn Example> {
    Box::new(CairoExample::new(
        options.to_proof_options(28, 8),
        trace_file_path,
        public_input_file_path,
    ))
}

pub struct CairoExample {
    options: ProofOptions,
    trace_file_path: String,
    public_input_file_path: String,
}

impl CairoExample {
    pub fn new(
        options: ProofOptions,
        trace_file_path: String,
        public_input_file_path: String,
    ) -> CairoExample {
        CairoExample {
            options,
            trace_file_path,
            public_input_file_path,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for CairoExample {
    fn prove(&self) -> StarkProof {
        debug!(
            "Generating a proof for running a Cairo program\n\
            ---------------------"
        );

        // read bytecode from file
        let file = File::open(&self.public_input_file_path).expect("Cannot open the file.");
        let reader = Mutex::new(BufReader::new(file));
        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode_length = usize::from_str(&line).unwrap();
        line.clear();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            2 * bytecode_length == bytecode.len(),
            "Wrong number of values provided."
        );
        line.clear();

        // read register boundary values
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let register_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            register_values.len() == 4,
            "Wrong number of register boundary values provided."
        );
        line.clear();

        // read rescue built-in pointer values
        reader.lock().unwrap().read_line(&mut line).unwrap();
        let rescue_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            rescue_pointer_values.len() == 2,
            "Wrong number of rescue pointer values provided."
        );

        // create a prover
        let prover = CairoProver::new(
            self.options.clone(),
            bytecode,
            register_values,
            rescue_pointer_values,
        );

        // generate execution trace
        let now = Instant::now();
        let trace = prover.build_trace_from_file(&self.trace_file_path);

        let trace_width = trace.width();
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace_width,
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // TODO: make it possible to print the custom trace
        // print_trace(&trace, 1, 0, 0..trace.width());

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        // read bytecode from file
        let file = File::open(&self.public_input_file_path).expect("Cannot open the file.");
        let reader = Mutex::new(BufReader::new(file));
        let mut line = String::new();
        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode_length = usize::from_str(&line).unwrap();
        line.clear();

        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let bytecode = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            2 * bytecode_length == bytecode.len(),
            "Wrong number of values provided."
        );
        line.clear();

        reader.lock().unwrap().read_line(&mut line).unwrap();
        line.pop();
        let register_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            register_values.len() == 4,
            "Wrong number of register boundary values provided."
        );
        line.clear();

        reader.lock().unwrap().read_line(&mut line).unwrap();
        let rescue_pointer_values = line
            .split([' '].as_ref())
            .map(|a| BaseElement::new(u64::from_str(&a).unwrap()))
            .collect::<Vec<BaseElement>>();
        assert!(
            rescue_pointer_values.len() == 2,
            "Wrong number of rescue pointer values provided."
        );

        // println!("{:#?}", bytecode);
        let pub_inputs = PublicInputs {
            bytecode: bytecode,
            register_values: register_values,
            rescue_pointer_values: rescue_pointer_values,
        };
        winterfell::verify::<CairoAir>(proof, pub_inputs)
    }

    //TODO: implement wrong trace checking
    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        Err(VerifierError::InconsistentBaseField)
    }
}
