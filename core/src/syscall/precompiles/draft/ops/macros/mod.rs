mod automod;
pub use automod::*;

macro_rules! process_generated {
    (
        #[path = "./"]
        mod __generated {$(
            #[path=$path:literal]
            mod $file_gm:ident;
            pub use $_tmp2_gm:ident as $gm:ident;
            use $_gm:ident as $mname:ident;
            const $_CNAME:ident: u8 = $bytecode:literal;
        )*}

    ) => {

        macro_rules! use_automod { () => {
            $(
                pub(crate) mod $file_gm;
                pub(crate) use $file_gm as $gm;
             )*
        }}
        pub(crate) use use_automod;

        #[macro_export]
        macro_rules! rwasm_selectors2 {
            ($$cb0:ident$$(::$$cbn:ident)* [$$($$rest:tt)*]) => {
                $$cb0$$(::$$cbn)*! {
                    [ $($mname)* ]
                    $$($$rest)*
                }
            }
        }
        pub use rwasm_selectors2;


    }
}
pub(crate) use process_generated;
