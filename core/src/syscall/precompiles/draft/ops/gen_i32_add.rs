use super::*;

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

impl OpcodeExecute<"I32Add"> for OpcodeExecuteBuilder {
    fn opcode_specific(&mut self) -> i32 {
        self.signed_x.wrapping_add(self.signed_y)
    }
}
