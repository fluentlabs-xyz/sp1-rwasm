pub use macro_rules_attribute::apply;

macro_rules! skip { ($($t:tt)*) => {} }

pub mod macros;
macros::use_automod!();

