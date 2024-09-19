use super::*;
use trace::*;

impl<'a, F: PrimeField32> OpcodeTrace<'a, "I32Add", F> for OpcodeTraceBuilder<'a, F> {
    fn opcode_specific(&mut self) {
        self.cols.is_add = F::from_bool(true);
        self.cols.riscv_opcode = F::from_canonical_u32(1); // 1 is the enum value of Opcode::Add
        let alu_event = AluEvent {
            lookup_id: create_alu_lookup_id(),
            shard: self.shard,
            channel: self.channel,
            clk: self.event.clk,
            opcode: Opcode::ADD,
            a: self.event.res_val,
            b: self.event.x_val,
            c: self.event.y_val,
            sub_lookups: create_alu_lookups(),
        };
        self.new_alu_events.push(alu_event);
    }
}

#[apply(skip)]
fn gen_execute() {
    let signed_x = x_val as i32;
    let signed_y = y_val as i32;
    (
        x_memory_read_record,
        y_memory_read_record,
        x_val,
        y_val,
        (signed_x.wrapping_add(signed_y)),
    )
}
