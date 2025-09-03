#[cfg(target_family = "wasm")]
#[path = "application_web.rs"]
mod application_web;

#[cfg(not(target_family = "wasm"))]
#[path = "application_native.rs"]
mod application_native;

pub use application_native::*;

mod asset_loader;
#[cfg(not(target_family = "wasm"))]
#[path = "framerate_handler_native.rs"]
mod framerate_handler_native;
#[cfg(target_family = "wasm")]
#[path = "framerate_handler_web.rs"]
mod framerate_handler_web;

pub use asset_loader::*;
