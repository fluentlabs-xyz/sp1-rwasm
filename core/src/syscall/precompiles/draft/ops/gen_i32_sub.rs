#[apply(skip)]
fn gen_trace() {
    cols.is_sub = F::from_bool(true);
    cols.riscv_opcode = F::from_canonical_u32(2); // 2 is the enum value of Opcode::SUB

    new_alu_events.push(AluEvent {
        lookup_id: create_alu_lookup_id(),
        shard,
        channel,
        clk: event.clk,
        opcode: Opcode::ADD,
        a: event.x_val,
        b: event.y_val,
        c: event.res_val,
        sub_lookups: create_alu_lookups(),
    })
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
        (signed_x.wrapping_sub(signed_y)),
    )
}
