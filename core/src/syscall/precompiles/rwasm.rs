
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! decl_rwasm_events {
    ($([$kind:ident [$([$CName:ident $byte:literal])+]])+) => { paste::paste! {$($(
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct [<$CName Event>];
    )*)*}};
}
crate::rwasm_chips!(decl_rwasm_events []);
 
#[macro_export]
macro_rules! decl_rwasm_chips {
    ($([$kind:ident [$([$CName:ident $byte:literal])+]])+) => { paste::paste! {$($(
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct [<$CName Chip>];
    )*)*}};
}
crate::rwasm_chips!(decl_rwasm_chips []);
 
