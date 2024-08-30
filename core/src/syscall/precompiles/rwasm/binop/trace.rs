use p3_field::PrimeField32;

use crate::memory::MemoryReadWriteCols;
use crate::utils::pad_rows;
use crate::{air::MachineAir, runtime::ExecutionRecord};

use super::columns::{BinOpCols, NUM_BINOP_MEM_COLS};
use super::BinOpChip;

use std::borrow::BorrowMut;


use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_maybe_rayon::prelude::{ParallelIterator, ParallelSlice};

use crate::bytes::event::ByteRecord;
use crate::{runtime::Program, stark::MachineRecord};




impl<F: PrimeField32> MachineAir<F> for BinOpChip {
    type Record = ExecutionRecord;
    type Program = Program;
    
    fn name(&self) -> String {
       "RwasmBinOp".to_string()
    }
    
    fn generate_trace(&self, input: &Self::Record, output: &mut Self::Record) -> RowMajorMatrix<F> {
        let mut rows = Vec::new();

        let mut new_byte_lookup_events = Vec::new();
        for i in 0..input.rwasm_binop_events.len() {
            let mut event = input.rwasm_binop_events[i].clone();
            let shard = event.shard;
            let channel = event.channel;

            let mut first_row = [F::zero(); NUM_BINOP_MEM_COLS];

            let first_cols: &mut BinOpCols<F> = first_row.as_mut_slice().borrow_mut();
            first_cols.res.populate(output, shard, channel, event.x_val[0], event.y_val[0]);
            for (i,read_record) in event.x_read_records.into_iter().enumerate(){
                first_cols.x_memory_record[i].populate(channel, read_record, &mut new_byte_lookup_events);

            }  

            for (i,read_record) in event.y_read_records.into_iter().enumerate(){
                first_cols.y_memory_record[i].populate(channel, read_record, &mut new_byte_lookup_events);
                
            }  

            first_cols.stack_ptr_record.populate(channel, event.stack_ptr_read_record, &mut new_byte_lookup_events);
            first_cols.x_addr=F::from_canonical_u32(event.x_addr);
            first_cols.y_addr= F::from_canonical_u32(event.y_addr);
            first_cols.is_real=F::from_bool(true);
          

          
            first_cols.pre_stack_ptr=F::from_canonical_u32(event.pre_stack_ptr_val);
            first_cols.post_stack_ptr=F::from_canonical_u32(event.post_stack_ptr_val);
            first_cols.stack_ptr_addr=F::from_canonical_u32(event.stack_ptr_addr);
           

            for (i,write_record) in event.res_write_records.into_iter().enumerate(){
                first_cols.y_write_record[i].populate(channel, write_record, &mut new_byte_lookup_events);
                
            }  

            first_cols.stack_ptr_write_record.populate(channel, event.stack_ptr_write_record, &mut new_byte_lookup_events);
            rows.push(first_row);

    }



    output.add_byte_lookup_events(new_byte_lookup_events);

    let num_real_rows = rows.len();

    pad_rows(&mut rows, || [F::zero(); NUM_BINOP_MEM_COLS]);
          // Convert the trace to a row major matrix.
          let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_BINOP_MEM_COLS,
        );

        // Write the nonces to the trace.
        for i in 0..trace.height() {
            let cols: &mut BinOpCols<F> = trace.values
                [i * NUM_BINOP_MEM_COLS..(i + 1) * NUM_BINOP_MEM_COLS]
                .borrow_mut();
            cols.nonce = F::from_canonical_usize(i);
        }

        trace
   }
    
    fn included(&self, shard: &Self::Record) -> bool {
       !shard.rwasm_binop_events.is_empty()
    }
}