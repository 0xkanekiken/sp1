use super::U256Field;
use crate::air::{MachineAir, SP1AirBuilder};
use crate::memory::{MemoryReadCols, MemoryWriteCols};
use crate::operations::field::field_op::{FieldOpCols, FieldOperation};
use crate::operations::field::params::Limbs;
use crate::runtime::{ExecutionRecord, Syscall, SyscallCode};
use crate::runtime::{MemoryReadRecord, MemoryWriteRecord};
use crate::stark::MachineRecord;
use crate::syscall::precompiles::SyscallContext;
use crate::utils::ec::field::{FieldParameters, NumLimbs};
use crate::utils::{limbs_from_prev_access, pad_rows};
use num::{BigUint, One};
use p3_air::{Air, BaseAir};
use p3_field::AbstractField;
use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::MatrixRowSlices;
use p3_maybe_rayon::prelude::{ParallelIterator, ParallelSlice};
use serde::{Deserialize, Serialize};
use sp1_derive::AlignedBorrow;
use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;

pub const NUM_UINT256_MUL_COLS: usize = size_of::<Uint256MulColumn<u8>>();

/// Number of `u32` words in a `BigUint` representing a 256 bit number.
pub const NUM_WORDS_IN_UINT256: usize = U256Field::NB_LIMBS / 4;

//***************************** Event ****************************/
//------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uint256MulEvent {
    pub shard: u32,
    pub clk: u32,
    pub x_ptr: u32,
    pub x: [u32; NUM_WORDS_IN_UINT256],
    pub y_ptr: u32,
    pub y: [u32; NUM_WORDS_IN_UINT256],
    pub x_memory_records: [MemoryWriteRecord; NUM_WORDS_IN_UINT256],
    pub y_memory_records: [MemoryReadRecord; NUM_WORDS_IN_UINT256],
}

//***************************** Chip ****************************/
//------------------------------------------------------------------------

#[derive(Default)]
pub struct Uint256MulChip;

impl Uint256MulChip {
    pub fn new() -> Self {
        Self
    }
}

//*****************************  Column ****************************/
//-----------------------------------------------------------------------

/// A set of columns for the Uint256Mul operation.
#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct Uint256MulColumn<T> {
    pub is_real: T,
    pub shard: T,
    pub clk: T,
    pub x_ptr: T,
    pub y_ptr: T,

    // Memory columns.
    pub x_memory: [MemoryWriteCols<T>; NUM_WORDS_IN_UINT256],
    pub y_memory: [MemoryReadCols<T>; NUM_WORDS_IN_UINT256],
    pub y_ptr_access: MemoryReadCols<T>,

    // Input values for the multiplication.
    pub x_input: [T; NUM_WORDS_IN_UINT256],
    pub y_input: [T; NUM_WORDS_IN_UINT256],

    // Output values.
    pub output: FieldOpCols<T, U256Field>,
}

impl<F: PrimeField32> MachineAir<F> for Uint256MulChip {
    type Record = ExecutionRecord;

    fn name(&self) -> String {
        "Uint256Mul".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // compute the number of events to process in each chunk.
        let chunk_size = std::cmp::max(input.uint256_mul_events.len() / num_cpus::get(), 1);

        // Generate the trace rows & corresponding records for each chunk of events concurrently.
        let rows_and_records = input
            .uint256_mul_events
            .par_chunks(chunk_size)
            .map(|events| {
                let mut records = ExecutionRecord::default();
                let mut new_byte_lookup_events = Vec::new();

                let rows = events
                    .iter()
                    .map(|event| {
                        let mut row: [F; NUM_UINT256_MUL_COLS] = [F::zero(); NUM_UINT256_MUL_COLS];
                        let cols: &mut Uint256MulColumn<F> = row.as_mut_slice().borrow_mut();

                        // Decode uint256 points
                        let x = biguint_from_words(&event.x);
                        let y = biguint_from_words(&event.y);

                        // Assign basic values to the columns.
                        {
                            cols.is_real = F::one();
                            cols.shard = F::from_canonical_u32(event.shard);
                            cols.clk = F::from_canonical_u32(event.clk);
                            cols.x_ptr = F::from_canonical_u32(event.x_ptr);
                            cols.y_ptr = F::from_canonical_u32(event.y_ptr);
                        }
                        // Memory columns.
                        {
                            // Populate the columns with the input values.
                            for i in 0..NUM_WORDS_IN_UINT256 {
                                // Populate the input_x columns.
                                cols.x_memory[i].populate(
                                    event.x_memory_records[i],
                                    &mut new_byte_lookup_events,
                                );
                                // Populate the input_y columns.
                                cols.y_memory[i].populate(
                                    event.y_memory_records[i],
                                    &mut new_byte_lookup_events,
                                );
                            }
                        }

                        // Populate the output columns for Uint256 multiplication.
                        cols.output.populate(&x, &y, FieldOperation::Mul);

                        row
                    })
                    .collect::<Vec<_>>();
                records.add_byte_lookup_events(new_byte_lookup_events);
                (rows, records)
            })
            .collect::<Vec<_>>();

        //  Generate the trace rows for each event.
        let mut rows = Vec::new();
        for (row, mut record) in rows_and_records {
            rows.extend(row);
            output.append(&mut record);
        }

        pad_rows(&mut rows, || [F::zero(); NUM_UINT256_MUL_COLS]);

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_UINT256_MUL_COLS,
        )
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.uint256_mul_events.is_empty()
    }
}

//************************ SYSCALL HANDLING  *****************************************
//------------------------------------------------------------------------------------

impl Syscall for Uint256MulChip {
    fn num_extra_cycles(&self) -> u32 {
        1
    }

    fn execute(&self, rt: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
        let start_clk = rt.clk;
        let x_ptr = arg1;
        if x_ptr % 4 != 0 {
            panic!();
        }
        let y_ptr = arg2;
        if y_ptr % 4 != 0 {
            panic!();
        }

        let x: [u32; NUM_WORDS_IN_UINT256] = rt
            .slice_unsafe(x_ptr, NUM_WORDS_IN_UINT256)
            .try_into()
            .unwrap();

        let (y_memory_records_vec, y_vec) = rt.mr_slice(y_ptr, NUM_WORDS_IN_UINT256);
        let y_memory_records = y_memory_records_vec.try_into().unwrap();
        let y: [u32; 8] = y_vec.try_into().unwrap();

        let uint256_x = biguint_from_words(&x);
        let uint256_y = biguint_from_words(&y);

        // Create a mask for the 256 bit number.
        let mask = BigUint::one() << 256;

        // Perform the multiplication and take the result modulo the mask.
        let result = (uint256_x * uint256_y) % mask;

        // Increment the clock as we are writing to the state.
        rt.clk += 1;

        // Convert the result to low endian u32 words.
        let result = biguint_to_words(&result);

        // write the state
        let state_memory_records = rt.mw_slice(x_ptr, &result).try_into().unwrap();

        let shard = rt.current_shard();

        rt.record_mut().uint256_mul_events.push(Uint256MulEvent {
            shard,
            clk: start_clk,
            x_ptr,
            x,
            y_ptr,
            y,
            x_memory_records: state_memory_records,
            y_memory_records,
        });

        None
    }
}

//****************************** AIR  ****************************************/
//-----------------------------------------------------------------------------

impl<F> BaseAir<F> for Uint256MulChip {
    fn width(&self) -> usize {
        NUM_UINT256_MUL_COLS
    }
}

impl<AB> Air<AB> for Uint256MulChip
where
    AB: SP1AirBuilder,
    Limbs<AB::Var, <U256Field as NumLimbs>::Limbs>: Copy,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Uint256MulColumn<AB::Var> = main.row_slice(0).borrow();

        let x = limbs_from_prev_access(&local.x_memory);
        let y = limbs_from_prev_access(&local.y_memory);

        let is_real = local.is_real;

        // Assert that is_real is a boolean.
        builder.assert_bool(is_real);

        // Evaluate the uint256 multiplication
        local
            .output
            .eval::<AB, _, _>(builder, &x, &y, FieldOperation::Mul);

        // Constraint the memory reads for the x and y values.
        builder.constraint_memory_access_slice(
            local.shard,
            local.clk.into(),
            local.y_ptr,
            &local.y_memory,
            local.is_real,
        );

        builder.constraint_memory_access_slice(
            local.shard,
            local.clk + AB::F::from_canonical_u32(1), // We read p at +1 since p, q could be the same.
            local.x_ptr,
            &local.x_memory,
            local.is_real,
        );

        builder.receive_syscall(
            local.shard,
            local.clk,
            AB::F::from_canonical_u32(SyscallCode::UINT256_MUL.syscall_id()),
            local.x_ptr,
            local.y_ptr,
            local.is_real,
        );
    }
}

//****************************** HELPER METHODS *******************************
//-----------------------------------------------------------------------------

// Convert a vector of u32 words into a BigUint
fn biguint_from_words(words: &[u32]) -> BigUint {
    let bytes = words
        .iter()
        .flat_map(|n| n.to_le_bytes())
        .collect::<Vec<_>>();
    BigUint::from_bytes_le(bytes.as_slice())
}

// Convert a BigUint into a vector of u32 words
fn biguint_to_words(value: &BigUint) -> [u32; NUM_WORDS_IN_UINT256] {
    let mut bytes = value.to_bytes_le();
    bytes.resize(NUM_WORDS_IN_UINT256 * 4, 0u8);

    let mut words = [0u32; NUM_WORDS_IN_UINT256];
    bytes
        .chunks_exact(4)
        .enumerate()
        .for_each(|(i, chunk)| words[i] = u32::from_le_bytes(chunk.try_into().unwrap()));
    words
}
