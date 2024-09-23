use super::*;

impl<'a, F: PrimeField32> OpcodeTrace<'a, "I32Xor", F> for OpcodeTraceBuilder<'a, F> {
    fn opcode_specific(&mut self) {
        //self.cols.bitop_selector.is_and = F::from_bool(true);
        self.cols.riscv_opcode = F::from_canonical_u32(37); 
        let alu_event = AluEvent{ 
            lookup_id:  create_alu_lookup_id(),
            shard: self.shard,
            channel: self.channel,
            clk: self.event.clk,
            opcode: Opcode::XOR,
            a: self.event.res_val, 
            b: self.event.x_val,
            c: self.event.y_val, 
            sub_lookups:  create_alu_lookups(),
        };
        self.new_alu_events.push(alu_event);
    }
}

impl OpcodeExecute<"I32Xor"> for OpcodeExecuteBuilder {
    fn opcode_specific(&mut self) -> i32 {
        self.signed_x ^ self.signed_y
    }
}
