#[cfg(target_family = "wasm")]
#[path = "application_web.rs"]
mod application_web;

#[cfg(not(target_family = "wasm"))]
#[path = "application_native.rs"]
mod application_native;

mod asset_loader;
// mod framerate_handler;

#[cfg(not(target_family = "wasm"))]
pub use application_native::*;
#[cfg(target_family = "wasm")]
pub use application_web::*;

pub use asset_loader::*;
