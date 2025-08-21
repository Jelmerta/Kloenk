#[cfg(target_family = "wasm")]
#[path = "run_web.rs"]
mod run_web;

#[cfg(not(target_family = "wasm"))]
#[path = "run_native.rs"]
mod run_native;

#[cfg(not(target_family = "wasm"))]
pub use run_native::*;
#[cfg(target_family = "wasm")]
pub use run_web::*;
