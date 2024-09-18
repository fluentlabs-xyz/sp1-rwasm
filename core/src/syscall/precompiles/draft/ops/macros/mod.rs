mod automod;
pub use automod::*;

macro_rules! process_generated {
    (
        #[path = "./"]
        mod __generated {$(
            #[path=$path:literal]
            mod $gm:ident;
        )*}

    ) => {

        macro_rules! use_automod { () => {
            $(pub(crate) mod $gm;)*
        }}
        pub(crate) use use_automod;

        #[macro_export]
        macro_rules! rwasm_selectors2 {
            ($$cb0:ident$$(::$$cbn:ident)* [$$($$rest:tt)*]) => {
                $$cb0$$(::$$cbn)*! {
                    [ $($gm)* ]
                    $$($$rest)*
                }
            }
        }
        pub use rwasm_selectors2;


    }
}
pub(crate) use process_generated;
