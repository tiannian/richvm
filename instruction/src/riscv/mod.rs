//! Instruction type for riscv

mod inst;
pub use inst::*;

mod prelude;
pub use prelude::*;

mod inst_r;
pub use inst_r::*;

mod inst_i;
pub use inst_i::*;

mod inst_s;
pub use inst_s::*;

mod inst_b;
pub use inst_b::*;

mod inst_u;
pub use inst_u::*;

mod inst_j;
pub use inst_j::*;

#[macro_export]
macro_rules! define_from_inner {
    ($inner: ty, $outer: ty) => {
        impl From<$inner> for $outer {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    };
}
