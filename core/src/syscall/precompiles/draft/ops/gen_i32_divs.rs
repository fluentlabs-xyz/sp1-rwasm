use super::*;

#[apply(skip)]
fn gen_trace() {
    cols.is_divs = F::from_bool(true);
    cols.riscv_opcode = F::from_canonical_u32(34); // 34 is the enum value of Opcode::DIVS

    new_alu_events.push(AluEvent {
        lookup_id: create_alu_lookup_id(),
        shard,
        channel,
        clk: event.clk,
        opcode: Opcode::DIV,
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
        (signed_x.wrapping_div(signed_y)),
    )
}
