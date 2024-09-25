#!/bin/sh -xe
//bin/sh -ec "test -f $0.sh" # Here we check that `.rs.sh` script exist, this script will replace module with updated version.
//bin/sh -ec "$0.sh; false"  # Now we just run the `.rs.sh` script, and return false every time, so our `sh` is stopped.

/*

This file can be used as script, directly.

You can run this file (as sh script) then `gen` module is added or removed.
It is not needed to rerun this script if just contents of `rs` file is changed.
Also it is compatible to run like `macros/automod.rs`, so from other folder (it will `cd` to correct dir).

Reason to do it:

Be compatible with rustfmt, modules can be formatted by this tool.
Using `macro` processing for each module to generate total matching function etc (pushing riscv code).

*/

use macro_rules_attribute::apply;

#[apply(super::process_generated)]
#[path = "./"]
mod __generated {
    // START MOD LIST
    #[path = "../gen_binop_0x67_i32_add.rs"]
    mod gen_binop_0x67_i32_add;
    pub use gen_binop_0x67_i32_add as gen_i32_add;
    use gen_i32_add as i32_add;
    const BYTECODE_i32_add: u8 = 0x67;
    #[path = "../gen_binop_0x68_i32_sub.rs"]
    mod gen_binop_0x68_i32_sub;
    pub use gen_binop_0x68_i32_sub as gen_i32_sub;
    use gen_i32_sub as i32_sub;
    const BYTECODE_i32_sub: u8 = 0x68;
    #[path = "../gen_binop_0x69_i32_mul.rs"]
    mod gen_binop_0x69_i32_mul;
    pub use gen_binop_0x69_i32_mul as gen_i32_mul;
    use gen_i32_mul as i32_mul;
    const BYTECODE_i32_mul: u8 = 0x69;
    #[path = "../gen_binop_0x6a_i32_divs.rs"]
    mod gen_binop_0x6a_i32_divs;
    pub use gen_binop_0x6a_i32_divs as gen_i32_divs;
    use gen_i32_divs as i32_divs;
    const BYTECODE_i32_divs: u8 = 0x6a;
    #[path = "../gen_binop_0x6b_i32_divu.rs"]
    mod gen_binop_0x6b_i32_divu;
    pub use gen_binop_0x6b_i32_divu as gen_i32_divu;
    use gen_i32_divu as i32_divu;
    const BYTECODE_i32_divu: u8 = 0x6b;
    #[path = "../gen_binop_0x6c_i32_rems.rs"]
    mod gen_binop_0x6c_i32_rems;
    pub use gen_binop_0x6c_i32_rems as gen_i32_rems;
    use gen_i32_rems as i32_rems;
    const BYTECODE_i32_rems: u8 = 0x6c;
    #[path = "../gen_binop_0x6d_i32_remu.rs"]
    mod gen_binop_0x6d_i32_remu;
    pub use gen_binop_0x6d_i32_remu as gen_i32_remu;
    use gen_i32_remu as i32_remu;
    const BYTECODE_i32_remu: u8 = 0x6d;
    #[path = "../gen_binop_0x6e_i32_and.rs"]
    mod gen_binop_0x6e_i32_and;
    pub use gen_binop_0x6e_i32_and as gen_i32_and;
    use gen_i32_and as i32_and;
    const BYTECODE_i32_and: u8 = 0x6e;
    #[path = "../gen_binop_0x6f_i32_or.rs"]
    mod gen_binop_0x6f_i32_or;
    pub use gen_binop_0x6f_i32_or as gen_i32_or;
    use gen_i32_or as i32_or;
    const BYTECODE_i32_or: u8 = 0x6f;
    #[path = "../gen_binop_0x70_i32_xor.rs"]
    mod gen_binop_0x70_i32_xor;
    pub use gen_binop_0x70_i32_xor as gen_i32_xor;
    use gen_i32_xor as i32_xor;
    const BYTECODE_i32_xor: u8 = 0x70;
    #[path = "../gen_binop_0x71_i32_shl.rs"]
    mod gen_binop_0x71_i32_shl;
    pub use gen_binop_0x71_i32_shl as gen_i32_shl;
    use gen_i32_shl as i32_shl;
    const BYTECODE_i32_shl: u8 = 0x71;
    #[path = "../gen_binop_0x72_i32_shrs.rs"]
    mod gen_binop_0x72_i32_shrs;
    pub use gen_binop_0x72_i32_shrs as gen_i32_shrs;
    use gen_i32_shrs as i32_shrs;
    const BYTECODE_i32_shrs: u8 = 0x72;
    #[path = "../gen_binop_0x73_i32_shru.rs"]
    mod gen_binop_0x73_i32_shru;
    pub use gen_binop_0x73_i32_shru as gen_i32_shru;
    use gen_i32_shru as i32_shru;
    const BYTECODE_i32_shru: u8 = 0x73;
    // END MOD LIST
}
