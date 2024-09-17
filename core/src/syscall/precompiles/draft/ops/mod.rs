use macro_rules_attribute::apply;

pub mod macros;
macros::use_automod!();

macro_rules! skip { ($($t:tt)*) => {} }
