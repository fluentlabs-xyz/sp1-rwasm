#[apply(skip)]
fn gen_trace() {
    cols.is_add = F::from_bool(true);
    cols.riscv_opcode = F::from_canonical_u32(1); // 1 is the enum value of Opcode::Add
    new_alu_events.push(AluEvent {
        lookup_id: create_alu_lookup_id(),
        shard,
        channel,
        clk: event.clk,
        opcode: Opcode::ADD,
        a: event.res_val,
        b: event.x_val,
        c: event.y_val,
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
        (signed_x.wrapping_add(signed_y)),
    )
}
