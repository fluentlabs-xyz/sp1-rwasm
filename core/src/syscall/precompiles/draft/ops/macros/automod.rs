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
    #[path = "../gen_i32_add.rs"]
    mod gen_i32_add;
    #[path = "../gen_i32_divs.rs"]
    mod gen_i32_divs;
    #[path = "../gen_i32_divu.rs"]
    mod gen_i32_divu;
    #[path = "../gen_i32_mul.rs"]
    mod gen_i32_mul;
    #[path = "../gen_i32_rems.rs"]
    mod gen_i32_rems;
    #[path = "../gen_i32_remu.rs"]
    mod gen_i32_remu;
    #[path = "../gen_i32_sub.rs"]
    mod gen_i32_sub;
    // END MOD LIST
}
