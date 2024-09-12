
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

}
