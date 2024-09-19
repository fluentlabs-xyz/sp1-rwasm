use super::*;
use trace::*;

impl<'a, F: PrimeField32> OpcodeTrace<'a, "I32DivS", F> for OpcodeTraceBuilder<'a, F> {
    fn opcode_specific(&mut self) {
        self.cols.is_divs = F::from_bool(true);
        self.cols.riscv_opcode = F::from_canonical_u32(34); // 34 is the enum value of Opcode::DIVS

        let alu_event = AluEvent {
            lookup_id: create_alu_lookup_id(),
            shard: self.shard,
            channel: self.channel,
            clk: self.event.clk,
            opcode: Opcode::DIV,
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
        (signed_x.wrapping_div(signed_y)),
    )
}
