#[cfg(target_arch = "wasm32")]
#[path = "run_web.rs"]
mod run;

#[cfg(not(target_arch = "wasm32"))]
#[path = "run_native.rs"]
mod run;

pub use run::*;
