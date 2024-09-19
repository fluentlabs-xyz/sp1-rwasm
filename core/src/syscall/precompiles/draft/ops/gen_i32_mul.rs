use super::*;
use trace::*;

impl<'a, F: PrimeField32> OpcodeTrace<'a, "I32Mul", F> for OpcodeTraceBuilder<'a, F> {
    fn opcode_specific(&mut self) {
        self.cols.is_mul = F::from_bool(true);
        self.cols.riscv_opcode = F::from_canonical_u32(30); // 30 is the enum value of Opcode::MUL

        let alu_event = AluEvent {
            lookup_id: create_alu_lookup_id(),
            shard: self.shard,
            channel: self.channel,
            clk: self.event.clk,
            opcode: Opcode::MUL,
            a: self.event.x_val,
            b: self.event.y_val,
            c: self.event.res_val,
            sub_lookups: create_alu_lookups(),
        };
        self.new_alu_events.push(alu_event);
    }
}

impl OpcodeExecute<"I32Mul"> for OpcodeExecuteBuilder {
    fn opcode_specific(&mut self) -> i32 {
        self.signed_x.wrapping_mul(self.signed_y)
    }
}
