use crate::air::{BaseAirBuilder, MachineAir, Polynomial, SP1AirBuilder, WORD_SIZE};
use crate::bytes::event::ByteRecord;
use crate::memory::{value_as_limbs, MemoryReadCols, MemoryWriteCols};
use crate::operations::field::field_op::{FieldOpCols, FieldOperation};
use crate::operations::field::params::NumWords;
use crate::operations::field::params::{Limbs, NumLimbs};
use crate::operations::IsZeroOperation;
use crate::runtime::{ExecutionRecord, Program, Syscall, SyscallCode};
use crate::runtime::{MemoryReadRecord, MemoryWriteRecord};
use crate::stark::MachineRecord;
use crate::syscall::precompiles::SyscallContext;
use crate::utils::ec::uint256::U256Field;
use crate::utils::{
    bytes_to_words_le, limbs_from_access, limbs_from_prev_access, pad_rows, words_to_bytes_le,
    words_to_bytes_le_vec,
};
use generic_array::GenericArray;
use num::Zero;
use num::{BigUint, One};
use p3_air::AirBuilder;
use p3_air::{Air, BaseAir};
use p3_field::AbstractField;
use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use serde::{Deserialize, Serialize};
use sp1_derive::AlignedBorrow;
use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;
use typenum::Unsigned;

/// The number of columns in the DraftCols.
const NUM_COLS: usize = size_of::<DraftCols<u8>>();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftEvent {
    pub lookup_id: usize,
    pub shard: u32,
    pub channel: u32,
    pub clk: u32,
    pub x_ptr: u32,
    pub x: Vec<u32>,
    pub y_ptr: u32,
    pub y: Vec<u32>,
    pub x_memory_records: Vec<MemoryWriteRecord>,
    pub y_memory_records: Vec<MemoryReadRecord>,
}

//#[derive(Default)]
pub struct DraftChip(SyscallCode);

impl DraftChip {
    pub const fn new(syscall_code: SyscallCode) -> Self {
        Self(syscall_code)
    }
}

//type WordsFieldElement = <U256Field as NumWords>::WordsFieldElement;
//const WORDS_FIELD_ELEMENT: usize = WordsFieldElement::USIZE;

const WORDS_FIELD_ELEMENT: usize = 8;

/// A set of columns for the Draft operation.
#[derive(Debug, Clone, AlignedBorrow)]
#[repr(C)]
pub struct DraftCols<T> {
    /// The shard number of the syscall.
    pub shard: T,

    /// The byte lookup channel.
    pub channel: T,

    /// The clock cycle of the syscall.
    pub clk: T,

    /// The none of the operation.
    pub nonce: T,

    /// The pointer to the first input.
    pub x_ptr: T,

    /// The pointer to the second input, which contains the y value and the modulus.
    pub y_ptr: T,

    // Memory columns.
    // x_memory is written to with the result, which is why it is of type MemoryWriteCols.
    pub x_memory: GenericArray<MemoryWriteCols<T>, typenum::U8>,
    pub y_memory: GenericArray<MemoryReadCols<T>, typenum::U8>,

    pub is_real: T,
}

impl<F: PrimeField32> MachineAir<F> for DraftChip {
    type Record = ExecutionRecord;
    type Program = Program;

    fn name(&self) -> String {
        "Draft".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Generate the trace rows & corresponding records for each chunk of events concurrently.
        let rows_and_records = input
            .draft_events
            .chunks(1)
            .map(|events| {
                let mut records = ExecutionRecord::default();
                let mut new_byte_lookup_events = Vec::new();

                let rows = events
                    .iter()
                    .map(|event| {
                        let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
                        let cols: &mut DraftCols<F> = row.as_mut_slice().borrow_mut();

                        // Decode u64 points
                        let x = BigUint::from_bytes_le(&words_to_bytes_le::<8>(&event.x));
                        let y = BigUint::from_bytes_le(&words_to_bytes_le::<8>(&event.y));

                        // Assign basic values to the columns.
                        cols.is_real = F::one();
                        cols.shard = F::from_canonical_u32(event.shard);
                        cols.channel = F::from_canonical_u32(event.channel);
                        cols.clk = F::from_canonical_u32(event.clk);
                        cols.x_ptr = F::from_canonical_u32(event.x_ptr);
                        cols.y_ptr = F::from_canonical_u32(event.y_ptr);

                        // Populate memory columns.
                        for i in 0..WORDS_FIELD_ELEMENT {
                            cols.x_memory[i].populate(
                                event.channel,
                                event.x_memory_records[i],
                                &mut new_byte_lookup_events,
                            );
                            cols.y_memory[i].populate(
                                event.channel,
                                event.y_memory_records[i],
                                &mut new_byte_lookup_events,
                            );
                        }

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

        pad_rows(&mut rows, || {
            let mut row: [F; NUM_COLS] = [F::zero(); NUM_COLS];
            let cols: &mut DraftCols<F> = row.as_mut_slice().borrow_mut();

            let x = BigUint::zero();
            let y = BigUint::zero();
            //cols.output
            //    .populate(&mut vec![], 0, 0, &x, &y, FieldOperation::Mul);

            row
        });

        // Convert the trace to a row major matrix.
        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut DraftCols<F> =
                trace.values[i * NUM_COLS..(i + 1) * NUM_COLS].borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        trace
    }

    fn included(&self, shard: &Self::Record) -> bool {
        !shard.draft_events.is_empty()
    }
}

impl Syscall for DraftChip {
    fn num_extra_cycles(&self) -> u32 {
        0
    }

    fn execute(&self, rt: &mut SyscallContext, arg1: u32, arg2: u32) -> Option<u32> {
        let x_ptr = arg1;
        if x_ptr % 8 != 0 {
            panic!();
        }
        let y_ptr = arg2;
        if y_ptr % 8 != 0 {
            panic!();
        }

        // First read the words for the x value. We can read a slice_unsafe here because we write
        // the computed result to x later.
        let x = rt.slice_unsafe(x_ptr, WORDS_FIELD_ELEMENT);

        // Read the y value.
        let (y_memory_records, y) = rt.mr_slice(y_ptr, WORDS_FIELD_ELEMENT);

        // Get the BigUint values for x, y, and the modulus.
        let u64_x = BigUint::from_bytes_le(&words_to_bytes_le_vec(&x));
        let u64_y = BigUint::from_bytes_le(&words_to_bytes_le_vec(&y));

        // Perform the multiplication and take the result modulo the modulus.
        let result: BigUint = u64_x * u64_y;

        let mut result_bytes = result.to_bytes_le();
        result_bytes.resize(8, 0u8); // Pad the result to 8 bytes.

        // Convert the result to little endian u32 words.
        let result = bytes_to_words_le::<8>(&result_bytes);

        // Write the result to x and keep track of the memory records.
        let x_memory_records = rt.mw_slice(x_ptr, &result);

        let lookup_id = rt.syscall_lookup_id;
        let shard = rt.current_shard();
        let channel = rt.current_channel();
        let clk = rt.clk;
        rt.record_mut().draft_events.push(DraftEvent {
            lookup_id,
            shard,
            channel,
            clk,
            x_ptr,
            x,
            y_ptr,
            y,
            x_memory_records,
            y_memory_records,
        });

        None
    }
}

impl<F> BaseAir<F> for DraftChip {
    fn width(&self) -> usize {
        NUM_COLS
    }
}

impl<AB> Air<AB> for DraftChip
where
    AB: SP1AirBuilder,
    //Limbs<AB::Var, <U256Field as NumLimbs>::Limbs>: Copy,
    //Limbs<AB::Var, 8>: Copy,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let local: &DraftCols<AB::Var> = (*local).borrow();
        let next = main.row_slice(1);
        let next: &DraftCols<AB::Var> = (*next).borrow();

        // Constrain the incrementing nonce.
        builder.when_first_row().assert_zero(local.nonce);
        builder
            .when_transition()
            .assert_eq(local.nonce + AB::Expr::one(), next.nonce);

        // We are computing (x * y) % modulus. The value of x is stored in the "prev_value" of
        // the x_memory, since we write to it later.
        let x_limbs = limbs_from_prev_access::<_, typenum::U8, _>(&local.x_memory);
        let y_limbs = limbs_from_access::<_, typenum::U8, _>(&local.y_memory);

        // Evaluate the u64 multiplication
/*
        local.output.eval(
            builder,
            &x_limbs,
            &y_limbs,
            FieldOperation::Mul,
            local.shard,
            local.channel,
            local.is_real,
        );
*/

/*
        // Assert that the correct result is being written to x_memory.
        builder
            .when(local.is_real)
            .assert_all_eq(local.output.result, value_as_limbs(&local.x_memory));
*/

        // Read and write x.
        builder.eval_memory_access_slice(
            local.shard,
            local.channel,
            local.clk.into(),
            local.x_ptr,
            &local.x_memory,
            local.is_real,
        );

        // Evaluate the y_ptr memory access. We concatenate y and modulus into a single array since
        // we read it contiguously from the y_ptr memory location.
        builder.eval_memory_access_slice(
            local.shard,
            local.channel,
            local.clk.into(),
            local.y_ptr,
            &[local.y_memory].concat(),
            local.is_real,
        );

        // Receive the arguments.
        builder.receive_syscall(
            local.shard,
            local.channel,
            local.clk,
            local.nonce,
            //AB::F::from_canonical_u32(SyscallCode::DRAFT.syscall_id()),
            AB::F::from_canonical_u32(self.0.syscall_id()),
            local.x_ptr,
            local.y_ptr,
            local.is_real,
        );

        // Assert that is_real is a boolean.
        builder.assert_bool(local.is_real);
    }
}
