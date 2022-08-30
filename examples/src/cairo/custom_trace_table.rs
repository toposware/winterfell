// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use core_utils::{collections::Vec, uninit_vector};
use winterfell::{
    math::{log2, FieldElement, StarkField},
    EvaluationFrame, Matrix, Trace, TraceInfo, TraceLayout,
};
// I think we shouldn't have to use it here. It should be passed with a parameter.
use super::{AUX_WIDTH, TRACE_WIDTH};

// RAP TRACE TABLE
// ================================================================================================
/// A concrete implementation of the [Trace] trait supporting custom RAPs.
///
/// This implementation should be sufficient for most use cases.
/// To create a trace table, you can use [RapTraceTable::new()] function, which takes trace
/// width and length as parameters. This function will allocate memory for the trace, but will not
/// fill it with data. To fill the execution trace, you can use the [fill()](RapTraceTable::fill)
/// method, which takes two closures as parameters:
///
/// 1. The first closure is responsible for initializing the first state of the computation
///    (the first row of the execution trace).
/// 2. The second closure receives the previous state of the execution trace as input, and must
///    update it to the next state of the computation.
///
/// You can also use [RapTraceTable::with_meta()] function to create a blank execution trace.
/// This function work just like [RapTraceTable::new()] function, but also takes a metadata
/// parameter which can be an arbitrary sequence of bytes up to 64KB in size.
pub struct RapTraceTable<B: StarkField> {
    layout: TraceLayout,
    trace: Matrix<B>,
    meta: Vec<u8>,
}

impl<B: StarkField> RapTraceTable<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new execution trace of the specified width and length.
    ///
    /// This allocates all the required memory for the trace, but does not initialize it. It is
    /// expected that the trace will be filled using one of the data mutator methods.
    ///
    /// # Panics
    /// Panics if:
    /// * `width` is zero or greater than 255.
    /// * `length` is smaller than 8, greater than biggest multiplicative subgroup in the field
    ///   `B`, or is not a power of two.
    pub fn new(width: usize, length: usize) -> Self {
        Self::with_meta(width, length, vec![])
    }

    /// Creates a new execution trace of the specified width and length, and with the specified
    /// metadata.
    ///
    /// This allocates all the required memory for the trace, but does not initialize it. It is
    /// expected that the trace will be filled using one of the data mutator methods.
    ///
    /// # Panics
    /// Panics if:
    /// * `width` is zero or greater than 255.
    /// * `length` is smaller than 8, greater than the biggest multiplicative subgroup in the
    ///   field `B`, or is not a power of two.
    /// * Length of `meta` is greater than 65535;
    pub fn with_meta(width: usize, length: usize, meta: Vec<u8>) -> Self {
        assert!(
            width > 0,
            "execution trace must consist of at least one column"
        );
        assert!(
            width <= TraceInfo::MAX_TRACE_WIDTH,
            "execution trace width cannot be greater than {}, but was {}",
            TraceInfo::MAX_TRACE_WIDTH,
            width
        );
        assert!(
            length >= TraceInfo::MIN_TRACE_LENGTH,
            "execution trace must be at least {} steps long, but was {}",
            TraceInfo::MIN_TRACE_LENGTH,
            length
        );
        assert!(
            length.is_power_of_two(),
            "execution trace length must be a power of 2"
        );
        assert!(
            log2(length) as u32 <= B::TWO_ADICITY,
            "execution trace length cannot exceed 2^{} steps, but was 2^{}",
            B::TWO_ADICITY,
            log2(length)
        );
        assert!(
            meta.len() <= TraceInfo::MAX_META_LENGTH,
            "number of metadata bytes cannot be greater than {}, but was {}",
            TraceInfo::MAX_META_LENGTH,
            meta.len()
        );

        let columns = unsafe { (0..width).map(|_| uninit_vector(length)).collect() };

        Self {
            layout: TraceLayout::new(width, [AUX_WIDTH], [3]),
            trace: Matrix::new(columns),
            meta,
        }
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Fill all rows in the execution trace.
    ///
    /// The rows are filled by executing the provided closures as follows:
    /// - `init` closure is used to initialize the first row of the trace; it receives a mutable
    ///   reference to the first state initialized to all zeros. The contents of the state are
    ///   copied into the first row of the trace after the closure returns.
    /// - `update` closure is used to populate all subsequent rows of the trace; it receives two
    ///   parameters:
    ///   - index of the last updated row (starting with 0).
    ///   - a mutable reference to the last updated state; the contents of the state are copied
    ///     into the next row of the trace after the closure returns.
    pub fn fill<I, U>(&mut self, init: I, update: U)
    where
        I: Fn(&mut [B]),
        U: Fn(usize, &mut [B]),
    {
        let mut state = vec![B::ZERO; self.main_trace_width()];
        init(&mut state);
        self.update_row(0, &state);

        for i in 0..self.length() - 1 {
            update(i, &mut state);
            self.update_row(i + 1, &state);
        }
    }

    /// Updates a single row in the execution trace with provided data.
    pub fn update_row(&mut self, step: usize, state: &[B]) {
        self.trace.update_row(step, state);
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of columns in this execution trace.
    pub fn width(&self) -> usize {
        self.main_trace_width()
    }

    /// Returns value of the cell in the specified column at the specified row of this trace.
    pub fn get(&self, column: usize, step: usize) -> B {
        self.trace.get(column, step)
    }

    /// Reads a single row from this execution trace into the provided target.
    pub fn read_row_into(&self, step: usize, target: &mut [B]) {
        self.trace.read_row_into(step, target);
    }
}

// TRACE TRAIT IMPLEMENTATION
// ================================================================================================

impl<B: StarkField> Trace for RapTraceTable<B> {
    type BaseField = B;

    fn layout(&self) -> &TraceLayout {
        &self.layout
    }

    fn length(&self) -> usize {
        self.trace.num_rows()
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        let next_row_idx = (row_idx + 1) % self.length();
        self.trace.read_row_into(row_idx, frame.current_mut());
        self.trace.read_row_into(next_row_idx, frame.next_mut());
    }

    fn main_segment(&self) -> &Matrix<B> {
        &self.trace
    }

    // The aux columns correspond to the (P_i) columns in the paper page 60.
    // First four columns are for offsets, last five for memory accesses.
    fn build_aux_segment<E>(
        &mut self,
        aux_segments: &[Matrix<E>],
        rand_elements: &[E],
    ) -> Option<Matrix<E>>
    where
        E: FieldElement<BaseField = Self::BaseField>,
    {
        // We only have one auxiliary segment.
        if !aux_segments.is_empty() {
            return None;
        }

        let mut current_row = unsafe { uninit_vector(self.width()) };
        // let mut next_row = unsafe { uninit_vector(self.width()) };
        self.read_row_into(0, &mut current_row);
        let mut aux_columns = vec![vec![E::ZERO; self.length()]; self.aux_trace_width()];

        // First initialize the P0.
        aux_columns[0][0] = (rand_elements[0] - current_row[16].into())
            / (rand_elements[0] - current_row[34].into());

        // Necessary variables: into() fails otherwise. Any better way to do this?
        let mut a: E = current_row[19].into();
        let mut a2: E = current_row[40].into();
        aux_columns[10][0] = (rand_elements[1] - (a + rand_elements[2] * current_row[20].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[41].into()));
        //println!("{}", aux_columns[0][0]);

        // Complete the first row.
        aux_columns[1][0] = aux_columns[0][0] * (rand_elements[0] - current_row[17].into())
            / (rand_elements[0] - current_row[35].into());
        aux_columns[2][0] = aux_columns[1][0] * (rand_elements[0] - current_row[18].into())
            / (rand_elements[0] - current_row[36].into());
        aux_columns[3][0] = aux_columns[2][0] * (rand_elements[0] - current_row[33].into())
            / (rand_elements[0] - current_row[37].into());
        aux_columns[4][0] = aux_columns[3][0] * (rand_elements[0] - current_row[52].into())
            / (rand_elements[0] - current_row[60].into());
        aux_columns[5][0] = aux_columns[4][0] * (rand_elements[0] - current_row[53].into())
            / (rand_elements[0] - current_row[61].into());
        aux_columns[6][0] = aux_columns[5][0] * (rand_elements[0] - current_row[54].into())
            / (rand_elements[0] - current_row[62].into());
        aux_columns[7][0] = aux_columns[6][0] * (rand_elements[0] - current_row[57].into())
            / (rand_elements[0] - current_row[63].into());
        aux_columns[8][0] = aux_columns[7][0] * (rand_elements[0] - current_row[58].into())
            / (rand_elements[0] - current_row[64].into());
        aux_columns[9][0] = aux_columns[8][0] * (rand_elements[0] - current_row[59].into())
            / (rand_elements[0] - current_row[65].into());

        a = current_row[21].into();
        a2 = current_row[42].into();
        aux_columns[11][0] = aux_columns[10][0]
            * (rand_elements[1] - (a + rand_elements[2] * current_row[22].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[43].into()));
        a = current_row[23].into();
        a2 = current_row[44].into();
        aux_columns[12][0] = aux_columns[11][0]
            * (rand_elements[1] - (a + rand_elements[2] * current_row[24].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[45].into()));
        a = current_row[25].into();
        a2 = current_row[46].into();
        aux_columns[13][0] = aux_columns[12][0]
            * (rand_elements[1] - (a + rand_elements[2] * current_row[26].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[47].into()));
        a = current_row[38].into();
        a2 = current_row[48].into();
        aux_columns[14][0] = aux_columns[13][0]
            * (rand_elements[1] - (a + rand_elements[2] * current_row[39].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[49].into()));
        a = current_row[50].into();
        a2 = current_row[55].into();
        aux_columns[15][0] = aux_columns[14][0]
            * (rand_elements[1] - (a + rand_elements[2] * current_row[51].into()))
            / (rand_elements[1] - (a2 + rand_elements[2] * current_row[56].into()));

        // Fill the rest of the rows. The last main trace row is filled with garbage, so we don't need to care about it.
        for index in 1..(self.length() - 1) {
            self.read_row_into(index, &mut current_row);
            aux_columns[0][index] = aux_columns[9][index - 1]
                * (rand_elements[0] - current_row[16].into())
                / (rand_elements[0] - current_row[34].into());
            aux_columns[1][index] = aux_columns[0][index]
                * (rand_elements[0] - current_row[17].into())
                / (rand_elements[0] - current_row[35].into());
            aux_columns[2][index] = aux_columns[1][index]
                * (rand_elements[0] - current_row[18].into())
                / (rand_elements[0] - current_row[36].into());
            aux_columns[3][index] = aux_columns[2][index]
                * (rand_elements[0] - current_row[33].into())
                / (rand_elements[0] - current_row[37].into());
            aux_columns[4][index] = aux_columns[3][index]
                * (rand_elements[0] - current_row[52].into())
                / (rand_elements[0] - current_row[60].into());
            aux_columns[5][index] = aux_columns[4][index]
                * (rand_elements[0] - current_row[53].into())
                / (rand_elements[0] - current_row[61].into());
            aux_columns[6][index] = aux_columns[5][index]
                * (rand_elements[0] - current_row[54].into())
                / (rand_elements[0] - current_row[62].into());
            aux_columns[7][index] = aux_columns[6][index]
                * (rand_elements[0] - current_row[57].into())
                / (rand_elements[0] - current_row[63].into());
            aux_columns[8][index] = aux_columns[7][index]
                * (rand_elements[0] - current_row[58].into())
                / (rand_elements[0] - current_row[64].into());
            aux_columns[9][index] = aux_columns[8][index]
                * (rand_elements[0] - current_row[59].into())
                / (rand_elements[0] - current_row[65].into());

            a = current_row[19].into();
            a2 = current_row[40].into();
            aux_columns[10][index] = aux_columns[15][index - 1]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[20].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[41].into()));
            a = current_row[21].into();
            a2 = current_row[42].into();
            aux_columns[11][index] = aux_columns[10][index]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[22].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[43].into()));
            a = current_row[23].into();
            a2 = current_row[44].into();
            aux_columns[12][index] = aux_columns[11][index]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[24].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[45].into()));
            a = current_row[25].into();
            a2 = current_row[46].into();
            aux_columns[13][index] = aux_columns[12][index]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[26].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[47].into()));
            a = current_row[38].into();
            a2 = current_row[48].into();
            aux_columns[14][index] = aux_columns[13][index]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[39].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[49].into()));
            a = current_row[50].into();
            a2 = current_row[55].into();
            aux_columns[15][index] = aux_columns[14][index]
                * (rand_elements[1] - (a + rand_elements[2] * current_row[51].into()))
                / (rand_elements[1] - (a2 + rand_elements[2] * current_row[56].into()));
        }

        Some(Matrix::new(aux_columns))
    }
}
