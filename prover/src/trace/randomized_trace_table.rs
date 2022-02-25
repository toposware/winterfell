// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use super::Trace;
use air::TraceInfo;
use math::{log2, StarkField};
use utils::{collections::Vec, uninit_vector};

#[cfg(not(feature = "concurrent"))]
use utils::collections::vec;

#[cfg(feature = "concurrent")]
use utils::{iterators::*, rayon};

// CONSTANTS
// ================================================================================================

const MIN_FRAGMENT_LENGTH: usize = 2;

// TRACE TABLE
// ================================================================================================
/// A concrete implementation of the [Trace] trait.
///
/// This implementation supports concurrent trace generation and should be sufficient for most use
/// cases. There are two ways to create a trace table trace.
///
/// First, you can use the [RandomizedTraceTable::init()] function which takes a set of vectors as a
/// parameter, where each vector contains values for a given column of the trace. This approach
/// allows you to build an execution trace as you see fit, as long as it meets a basic set of
/// requirements. These requirements are:
///
/// 1. Lengths of all columns in the execution trace must be the same.
/// 2. The length of the columns must be some power of two.
///
/// The other approach is to use [RandomizedTraceTable::new()] function, which takes trace width and
/// length as parameters. This function will allocate memory for the trace, but will not fill it
/// with data. To fill the execution trace, you can use the [fill()](RandomizedTraceTable::fill) method,
/// which takes two closures as parameters:
///
/// 1. The first closure is responsible for initializing the first state of the computation
///    (the first row of the execution trace).
/// 2. The second closure receives the previous state of the execution trace as input, and must
///    update it to the next state of the computation.
///
/// You can also use [RandomizedTraceTable::with_meta()] function to create a blank execution trace.
/// This function work just like [RandomizedTraceTable::new()] function, but also takes a metadata
/// parameter which can be an arbitrary sequence of bytes up to 64KB in size.
///
/// # Concurrent trace generation
/// For computations which consist of many small independent computations, we can generate the
/// execution trace of the entire computation by building fragments of the trace in parallel,
/// and then joining these fragments together.
///
/// For this purpose, `RandomizedTraceTable` struct exposes [fragments()](RandomizedTraceTable::fragments)
/// method, which takes fragment length as a parameter, breaks the execution trace into equally
/// sized fragments, and returns an iterator over these fragments. You can then use fragment's
/// [fill()](RandomizedTraceTableFragment::fill) method to fill all fragments with data in parallel.
/// The semantics of the fragment's [RandomizedTraceTableFragment::fill()] method are identical to the
/// semantics of the [RandomizedTraceTable::fill()] method.
pub struct RandomizedTraceTable<B: StarkField> {
    trace: Vec<Vec<B>>,
    aux_columns: Vec<Vec<B>>,
    aux_init: Fn(&[B], &[B], &mut [B]),
    aux_update: Fn(usize, &[B], &[B], &mut [B]),
    ncoins: usize,
    finished: bool,
    meta: Vec<u8>,
}

impl<B: StarkField> RandomizedTraceTable<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new execution trace of the specified width and length as well as number of
    /// auxiliary columns for performing computations using up to `ncoins` random field elements,
    /// typically used for copy constraints.
    /// This allocates all the required memory for the trace, but does not initialize it. It is
    /// expected that the trace will be filled using one of the data mutator methods.
    ///
    /// # Panics
    /// Panics if:
    /// * `width` + `aux_width` is zero or greater than 255.
    /// * `length` is smaller than 8, greater than biggest multiplicative subgroup in the field
    ///   `B`, or is not a power of two.
    /// 
    
    // TODO: add some check for ncoins
    pub fn new(width: usize, aux_width: usize, length: usize, ncoins: usize) -> Self {
        Self::with_meta(width, aux_width, length, ncoins, vec![])
    }

    /// Creates a new execution trace of the specified width and length and number of auxiliary
    /// columns, and with the specified metadata.
    ///
    /// This allocates all the required memory for the trace, but does not initialize it. It is
    /// expected that the trace will be filled using one of the data mutator methods.
    ///
    /// # Panics
    /// Panics if:
    /// * `width` + `aux_width` is zero or greater than 255.
    /// * `length` is smaller than 8, greater than the biggest multiplicative subgroup in the
    ///   field `B`, or is not a power of two.
    /// * Length of `meta` is greater than 65535;
    pub fn with_meta(width: usize, aux_width: usize, length: usize, ncoins: usize, meta: Vec<u8>) -> Self {
        assert!(
            width > 0,
            "execution trace must consist of at least one register"
        );
        assert!(
            width + aux_width <= TraceInfo::MAX_TRACE_WIDTH,
            "execution trace width cannot be greater than {}, but was {}",
            TraceInfo::MAX_TRACE_WIDTH,
            width
        );
        assert!(
            length >= TraceInfo::MIN_TRACE_LENGTH,
            "execution trace must be at lest {} steps long, but was {}",
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

        let registers = unsafe { (0..width).map(|_| uninit_vector(length)).collect() };
        let aux_registers = unsafe { (0..aux_width).map(|_| uninit_vector(length)).collect() };
        Self {
            trace: registers,
            aux_columns: aux_registers,
            ncoins: ncoins,
            finished: false,
            meta,
        }
    }

    /// Creates a new execution trace from a list of provided register traces.
    ///
    /// The provided `registers` and `aux_registers` vector are expected to
    /// contain register traces.
    ///
    /// # Panics
    /// Panics if:
    /// * The `registers` vector is empty or `registers` concatenated with
    ///   `aux_registers`has over 255 registers.
    /// * Number of elements in any of the registers is smaller than 8,
    ///   greater than the biggest multiplicative subgroup in the field `B`,
    ///   or is not a power of two.
    /// * Number of elements is not identical for all registers or auxiliary register.
    pub fn init(registers: Vec<Vec<B>>, aux_registers: Vec<Vec<B>, ncoins:usize) -> Self {
        assert!(
            !registers.is_empty(),
            "execution trace must consist of at least one register"
        );
        assert!(
            registers.len() + aux_registers.len() <= TraceInfo::MAX_TRACE_WIDTH,
            "execution trace width cannot be greater than {}, but was {}",
            TraceInfo::MAX_TRACE_WIDTH,
            registers.len() + aux_registers.len()
        );
        let trace_length = registers[0].len();
        assert!(
            trace_length >= TraceInfo::MIN_TRACE_LENGTH,
            "execution trace must be at lest {} steps long, but was {}",
            TraceInfo::MIN_TRACE_LENGTH,
            trace_length
        );
        assert!(
            trace_length.is_power_of_two(),
            "execution trace length must be a power of 2"
        );
        assert!(
            log2(trace_length) as u32 <= B::TWO_ADICITY,
            "execution trace length cannot exceed 2^{} steps, but was 2^{}",
            B::TWO_ADICITY,
            log2(trace_length)
        );
        for register in registers.iter() {
            assert_eq!(
                register.len(),
                trace_length,
                "all register traces must have the same length"
            );
        }
        for register in aux_registers.iter() {
            assert_eq!(
                register.len(),
                trace_length,
                "all register traces must have the same length"
            );
        }

        Self {
            trace: registers,
            aux_columns: aux_registers,
            ncois: ncoins,
            finished: aux_registers.is_empty(),
            meta: vec![],
        }
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Updates a value in a single cell of the execution trace.
    ///
    /// Specifically, the value in the specified `register` and the specified `step` is set to the
    /// provide `value`.
    ///
    /// # Panics
    /// Panics if either `register` or `step` are out of bounds for this execution trace.
    pub fn set(&mut self, register: usize, step: usize, value: B) {
        if(register >= trace.width()){
            self.aux_columns[register - trace.width()][step] = value;
        }
        self.trace[register][step] = value;
    }

    /// Updates metadata for this execution trace to the specified vector of bytes.
    ///
    /// # Panics
    /// Panics if the length of `meta` is greater than 65535;
    pub fn set_meta(&mut self, meta: Vec<u8>) {
        assert!(
            meta.len() <= TraceInfo::MAX_META_LENGTH,
            "number of metadata bytes cannot be greater than {}, but was {}",
            TraceInfo::MAX_META_LENGTH,
            meta.len()
        );
        self.meta = meta
    }

    /// Fill all rows in the execution trace.
    ///
    /// The rows are filled by executing the provided closures as follows:
    /// - `init` closure is used to initialize the first row of the trace (b); it receives a mutable
    ///   reference to the first state initialized to all zeros. The contents of the state are
    ///   copied into the first row of the trace after the closure returns.
    /// - `aux_init` closure is used to initialize the first row of the auxiliary columns; 
    ///   it receives references to a vector of random values and to the first row of the trace, as well as
    ///   a mutable reference to the first row of the auxiliary columns initialized to all zeros.
    ///   The contents of the state are copied into the first row of the auxiliary columns after the
    ///   closure returns.
    /// - `update` closure is used to populate all subsequent rows of the trace; it receives two
    ///   parameters:
    ///   - index of the last updated row (starting with 0).
    ///   - a mutable reference to the last updated state; the contents of the state are copied
    ///     into the next row of the trace after the closure returns.
    /// - `aux_columns` closure is used to populate all the subsequent rows of the auxiliary columns;
    ///   it receives four parameters:
    ///   - index of the last updated row (starting with 0).
    ///   - a reference to a vector of (random) field elements.
    ///   - a reference to the current row trace (index + 1)
    ///   - a mutable reference to the last updated state; the contents of the state are copied
    ///     into the next row of the auxiliary columns after the closure returns.
    pub fn fill<I, U>(&mut self, init: I, update: U, aux_init: J, aux_update: R)
    where
        I: Fn(&mut [B]),
        U: Fn(usize, &mut [B]),
        J: Fn(&[B], &[B], &mut[B]),
        R: Fn(usize, &[B], &[B], &mut[B])
    {
        let mut state = vec![B::ZERO; self.width()];
        init(&mut state);
        self.update_row(0, &state);

        for i in 0..self.length() - 1 {
            update(i, &mut state);
            self.update_row(i + 1, &state);
        }

        // Lazy evaluation of auxiliary columns
        self.aux_init = aux_init;
        self.aux_update = aux_update;
    }

    /// Updates a single row in the execution trace with provided data.
    pub fn update_row(&mut self, step: usize, state: &[B]) {
        for (register, &value) in self.trace.iter_mut().zip(state) {
            register[step] = value;
        }
    }

    /// Updates a single row in the auxiliary columns with provided data.
    pub fn update_aux_row(&mut self, step: usize, state: &[B]) {
        for (register, &value) in self.aux_columns.iter_mut().zip(state) {
            register[step] = value;
        }
    }

    // FRAGMENTS
    // --------------------------------------------------------------------------------------------

    /// Breaks the execution trace into mutable fragments.
    ///
    /// The number of rows in each fragment will be equal to `fragment_length` parameter. The
    /// returned fragments can be used to update data in the trace from multiple threads.
    ///
    /// # Panics
    /// Panics if `fragment_length` is smaller than 2, greater than the length of the trace,
    /// or is not a power of two.
    #[cfg(not(feature = "concurrent"))]
    pub fn fragments(&mut self, fragment_length: usize) -> vec::IntoIter<RandomizedTraceTableFragment<B>> {
        self.build_fragments(fragment_length).into_iter()
    }

    /// Breaks the execution trace into mutable fragments.
    ///
    /// The number of rows in each fragment will be equal to `fragment_length` parameter. The
    /// returned fragments can be used to update data in the trace from multiple threads.
    ///
    /// # Panics
    /// Panics if `fragment_length` is smaller than 2, greater than the length of the trace,
    /// or is not a power of two.
    #[cfg(feature = "concurrent")]
    pub fn fragments(
        &mut self,
        fragment_length: usize,
    ) -> rayon::vec::IntoIter<RandomizedTraceTableFragment<B>> {
        self.build_fragments(fragment_length).into_par_iter()
    }

    /// Returns a vector of trace fragments each covering the number of steps specified by the
    /// `fragment_length` parameter.
    fn build_fragments(&mut self, fragment_length: usize) -> Vec<RandomizedTraceTableFragment<B>> {
        assert!(
            fragment_length >= MIN_FRAGMENT_LENGTH,
            "fragment length must be at least {}, but was {}",
            MIN_FRAGMENT_LENGTH,
            fragment_length
        );
        assert!(
            fragment_length <= self.length(),
            "length of a fragment cannot exceed {}, but was {}",
            self.length(),
            fragment_length
        );
        assert!(
            fragment_length.is_power_of_two(),
            "fragment length must be a power of 2"
        );
        let num_fragments = self.length() / fragment_length;

        let mut fragment_data = (0..num_fragments).map(|_| Vec::new()).collect::<Vec<_>>();
        self.trace.iter_mut().for_each(|column| {
            for (i, fragment) in column.chunks_mut(fragment_length).enumerate() {
                fragment_data[i].push(fragment);
            }
        });

        fragment_data
            .into_iter()
            .enumerate()
            .map(|(i, data)| RandomizedTraceTableFragment {
                index: i,
                offset: i * fragment_length,
                data,
            })
            .collect()
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the entire register trace for the register at the specified index.
    pub fn get_register(&self, idx: usize) -> &[B] {
        if idx > self.width() {
            &self.aux_columns[idx - self.width()]
        }
        &self.trace[idx]
    }
}

// TRACE TRAIT IMPLEMENTATION
// ================================================================================================

impl<B: StarkField> Trace for RandomizedTraceTable<B> {
    type BaseField = B;

    fn width(&self) -> usize {
        self.trace.len()
    }

    fn aux_columns_width(&self) -> usize {
        self.aux_columns.len()
    }

    fn length(&self) -> usize {
        self.trace[0].len()
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn get(&self, register: usize, step: usize) -> B {
        if register > self.width() {
            self.aux_columns[register - self.width()][step]
        }
        self.trace[register][step]
    }

    fn read_row_into(&self, step: usize, target: &mut [B]) {
        for (i, register) in self.trace.iter().enumerate() {
            target[i] = register[step];
        }
    }

    fn get_columns(&self) -> Vec<Vec<B>> {
        self.trace.clone()
    }

    // is actually computing the auxiliary columns 
    fn set random_coeffs(&mut self, coeffs: Vec<B>) {
        let mut state = vec![B::ZERO; self.aux_columns_width()];
        let mut trace_state = vec![B::ZERO; self.width()];
        self.read_row_into(0, &trace_state);
        self.aux_init(&coeffs, &trace_state, &mut state);
        self.update_aux_row(0, &state);

        for i in 0..self.length() - 1 {
            self.read_row_into(i+1, &trace_state);
            self.aux_update(&coeffs, &trace_state, &mut state);
            self.update_aux_row(i + 1, &state);
        }
        self.finished = true;
    }

    fn number_of_coins(&self) -> usize {
        self.ncoins
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_aux_columns(self) -> Vec<Vec<B>> {
        self.aux_columns
    }
}

// TRACE FRAGMENTS
// ================================================================================================
/// A set of consecutive rows of an execution trace.
///
/// An execution trace fragment is a "view" into the specific execution trace. Updating data in
/// the fragment, directly updates the data in the underlying execution trace.
///
/// A fragment cannot be instantiated directly but is created by executing
/// [RandomizedTraceTable::fragments()] method.
///
/// A fragment always contains contiguous rows, and the number of rows is guaranteed to be a power
/// of two.
pub struct RandomizedTraceTableFragment<'a, B: StarkField> {
    index: usize,
    offset: usize,
    data: Vec<&'a mut [B]>,
}

impl<'a, B: StarkField> RandomizedTraceTableFragment<'a, B> {
    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the index of this fragment.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the step at which the fragment starts in the context of the original execution
    /// trace.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the number of rows in this execution trace fragment.
    pub fn length(&self) -> usize {
        self.data[0].len()
    }

    /// Returns the width of the fragment (same as the width of the underlying execution trace).
    pub fn width(&self) -> usize {
        self.data.len()
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Fills all rows in the fragment.
    ///
    /// The rows are filled by executing the provided closures as follows:
    /// - `init` closure is used to initialize the first row of the fragment; it receives a
    ///   mutable reference to the first state initialized to all zeros. Contents of the state are
    ///   copied into the first row of the fragment after the closure returns.
    /// - `update` closure is used to populate all subsequent rows of the fragment; it receives two
    ///   parameters:
    ///   - index of the last updated row (starting with 0).
    ///   - a mutable reference to the last updated state; the contents of the state are copied
    ///     into the next row of the fragment after the closure returns.
    pub fn fill<I, T>(&mut self, init_state: I, update_state: T)
    where
        I: Fn(&mut [B]),
        T: Fn(usize, &mut [B]),
    {
        let mut state = vec![B::ZERO; self.width()];
        init_state(&mut state);
        self.update_row(0, &state);

        for i in 0..self.length() - 1 {
            update_state(i, &mut state);
            self.update_row(i + 1, &state);
        }
    }

    /// Updates a single row in the fragment with provided data.
    pub fn update_row(&mut self, row_idx: usize, row_data: &[B]) {
        for (column, &value) in self.data.iter_mut().zip(row_data) {
            column[row_idx] = value;
        }
    }
}
