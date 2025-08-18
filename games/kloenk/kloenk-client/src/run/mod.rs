#[cfg(target_arch = "wasm32")]
#[path = "run_web.rs"]
mod run_web;

#[cfg(not(target_arch = "wasm32"))]
#[path = "run_native.rs"]
mod run_native;

#[cfg(not(target_arch = "wasm32"))]
pub use run_native::*;
#[cfg(target_arch = "wasm32")]
pub use run_web::*;
