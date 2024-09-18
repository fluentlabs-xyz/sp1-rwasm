
#[macro_export]
macro_rules! rwasm_chips {
    ($cb0:ident$(::$cbn:ident)* [$($rest:tt)*]) => {
        $cb0$(::$cbn)*! {
            [binop [
                [I32Add  0x67]
                [I32Sub  0x68]
                [I32Mul  0x69]
                [I32DivS 0x6a]
                [I32DivU 0x6b]
                [I32RemS 0x6c]
                [I32RemU 0x6d]
                [I32And  0x6e]
                [I32Or   0x6f]
                [I32Xor  0x70]
            ]]
            /* TODO
              - rotl
              - rotr
              - eq
              - ne
              - lt_s
              - lt_u
              - gt_s
              - gt_u
              - le_s
              - le_u
              - ge_s
              - ge_u
            */
            $($rest)*
        }
    }
}

#[macro_export]
macro_rules! append_rwasm_record_events {
    (
        $([$kind:ident [$([$CName:ident $byte:literal])*]])*
        $(#[$($attr:tt)*])+
        pub struct $SName:ident {
            $($code:tt)*
        }
    ) => { paste::paste! {
        $(#[$($attr)*])*
        pub struct $SName {
            $($code)*
            pub draft_events: Vec<DraftEvent>,
        }
    }};

    ($($rest:tt)*) => { $crate::rwasm_chips! { $crate::append_rwasm_record_events [$($rest)*] } };
}

#[macro_export]
macro_rules! append_rwasm_syscall_code {
    (
        $([$kind:ident [$([$CName:ident $byte:literal])+]])+
        $(#[$($attr:tt)*])+
        pub enum $EName:ident {
            $($code:tt)*
        }
    ) => { paste::paste! {
        $(#[$($attr)*])*
        pub enum $EName {
            $($code)*
            RWASM_00        = 0x01_01_01_00,
            // Last byte is rwasm bytecode byte of instruction
            $($([<RWASM_ $CName:snake:upper>] = 0x01_01_01_00 + $byte,)*)*
            RWASM_END       = 0x01_ff_ff_ff,
        }
    }};

    ($($rest:tt)*) => { $crate::rwasm_chips! { $crate::append_rwasm_syscall_code [$($rest)*] } };
}


/*
#[macro_export]
macro_rules! append_rwasm_chip_variants {
    (
        $([$kind:ident [$([$CName:ident $byte:literal])+]])+
        $(#[$($attr:tt)*])+
        pub enum $EName:ident {
            $($code:tt)*
        }
    ) => { paste::paste! {
        $(#[$($attr)*])*
        pub enum $EName {
            $($code)*
            $($([<$CName Chip>]([<$CName Chip>]),)*)*
        }
    }};

    ($($rest:tt)*) => { $crate::rwasm_chips! { $crate::append_rwasm_chip_variants [$($rest)*] } };
}
*/

#[macro_export]
macro_rules! decl_rwasm_template {

    ($(#[$($attr:tt)*])+ pub struct $Name:ident$(<$($G:ident),*>)? { $($code:tt)* }) => { paste::paste! {
        macro_rules! [<rwasm_template_ $Name:snake>] {
            ($$($$rest:tt)*) => { $(#[$($attr)*])+ pub struct $Name$(<$($G),*>)? { $($code)* $$($$rest)* } };
        }
        pub(crate) use [<rwasm_template_ $Name:snake>];
    }};

    ($(#[$($attr:tt)*])+ pub enum $Name:ident { $($code:tt)* }) => { paste::paste! {
        macro_rules! [<rwasm_template_ $Name:snake>] {
            ($$($$rest:tt)*) => { $(#[$($attr)*])+ pub enum $Name { $($code)* $$($$rest)* } };
        }
        pub(crate) use [<rwasm_template_ $Name:snake>];
    }};


    //(impl Syscall for $Name:ident {
    (impl Syscall for BinOp32Chip {

         fn num_extra_cycles(&$self1:ident) -> u32 { $($code1:tt)* }

         fn execute(&$self:ident, $rt:ident: &mut SyscallContext, $arg1:ident: u32, $arg2:ident: u32) -> Option<u32> {
             {$($code2:tt)*}
             let ($($bind:tt)*) = { { $($code3:tt)* } };
             {$($code4:tt)*}
         }


      }) => { paste::paste! {
        //macro_rules! [<rwasm_template_impl_syscall_ $Name:snake>] {
        macro_rules! [<rwasm_template_impl_syscall_bin_op32_chip>] {
            ($$($$rest:tt)*) => {

         //impl Syscall for $Name {
         impl Syscall for BinOp32Chip {

         fn num_extra_cycles(&$self1) -> u32 { $($code1)* }

         fn execute(&$self, $rt: &mut SyscallContext, $arg1: u32, $arg2: u32) -> Option<u32> {
             $($code2)*
             let ($($bind)*) = { $($code3)* $$($$rest)* };
             $($code4)*
         }

         }


            };
        }
        //pub(crate) use [<rwasm_template_impl_syscall_ $Name:snake>];
        pub(crate) use [<rwasm_template_impl_syscall_bin_op32_chip>];
    }};



    //(impl $Name:ident {
    (impl BinOp32Chip {

         fn event_to_row <F: PrimeField32>(
            &$self:ident,
            $event:ident: &BinOp32Event,
         ) -> (
            [F; NUM_BINOP32_MEM_COLS],
            Vec<ByteLookupEvent>$(,)?
         ) { {$($code1:tt)*} {$($code2:tt)*} }

      }) => { paste::paste! {
        //macro_rules! [<rwasm_template_impl_ $Name:snake>] {
        macro_rules! [<rwasm_template_impl_bin_op32_chip>] {
            ($cols:ident $$($$rest:tt)*) => {

         //impl $Name {
         impl BinOp32Chip {

         fn event_to_row <F: PrimeField32>(
            &$self,
            $event: &BinOp32Event,
         ) -> (
            [F; NUM_BINOP32_MEM_COLS],
            Vec<ByteLookupEvent>,
         ) {
               $($code1)*
               $$($$rest)*
               $($code2)*
           }

         }

            };
        }
        //pub(crate) use [<rwasm_template_impl_ $Name:snake>];
        pub(crate) use [<rwasm_template_impl_bin_op32_chip>];
    }};



}

#[macro_export]
macro_rules! rwasm_selectors {
    ($cb0:ident$(::$cbn:ident)* [$($rest:tt)*]) => {
        $cb0$(::$cbn)*! {
            [ add sub mul divu divs remu rems ]
            $($rest)*
        }
    }
}

#[macro_export]
macro_rules! append_rwasm_selectors {
    (
        [$($selector:ident)*]
        $(#[$($attr:tt)*])+
        pub(crate) struct $Name:ident$(<$($G:ident),*>)? {
            $($code:tt)*
        }
    ) => { paste::paste! {
        $(#[$($attr)*])*
        pub(crate) struct $Name$(<$($G),*>)? {
            $($code)*
            $(pub [<is_ $selector>]: T,)*
        }
    }};

    //($($rest:tt)*) => { $crate::syscall::precompiles::draft::ops::macros::rwasm_selectors! { $crate::append_rwasm_selectors [$($rest)*] } };
    ($($rest:tt)*) => { $crate::rwasm_selectors! { $crate::append_rwasm_selectors [$($rest)*] } };
}
