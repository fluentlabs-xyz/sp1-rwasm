#[apply(skip)]
fn gen_trace() {
    cols.is_divu = F::from_bool(true);
    cols.riscv_opcode = F::from_canonical_u32(35); // 34 is the enum value of Opcode::DIVU

    new_alu_events.push(AluEvent {
        lookup_id: create_alu_lookup_id(),
        shard,
        channel,
        clk: event.clk,
        opcode: Opcode::DIVU,
        a: event.res_val,
        b: event.x_val,
        c: event.y_val,
        sub_lookups: create_alu_lookups(),
    })
}

#[apply(skip)]
fn gen_execute() {
    todo!()
}
