#[cfg(target_arch = "wasm32")]
#[path = "application_web.rs"]
mod application_web;

#[cfg(not(target_arch = "wasm32"))]
#[path = "application_native.rs"]
mod application_native;

mod asset_loader;

#[cfg(not(target_arch = "wasm32"))]
pub use application_native::*;
#[cfg(target_arch = "wasm32")]
pub use application_web::*;

pub use asset_loader::*;
