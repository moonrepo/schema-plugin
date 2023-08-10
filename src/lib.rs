// WASM cannot be executed through the test runner and we need to avoid building
// WASM code for non-WASM targets. We can solve both of these with a cfg flag.

mod proto;

pub use proto::*;
