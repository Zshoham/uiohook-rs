#![cfg_attr(rustdoc, feature(doc_cfg))]

mod error;
pub use error::*;

pub mod hook;
pub mod system_properties;

#[doc(inline)]
pub use hook::event::{EventKind, EventMetaData, EventType, HookEvent};
#[doc(inline)]
pub use hook::global::hook_start;
#[doc(inline)]
pub use hook::global::hook_start_blocking;
#[doc(inline)]
pub use hook::Hook;

mod macros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
