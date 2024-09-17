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

    }
}
pub(crate) use process_generated;
