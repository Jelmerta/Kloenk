#[cfg(target_family = "wasm")]
#[path = "application_web.rs"]
mod application_web;

#[cfg(not(target_family = "wasm"))]
#[path = "application_native.rs"]
mod application_native;

#[cfg(not(target_family = "wasm"))]
pub use application_native::*;
#[cfg(target_family = "wasm")]
pub use application_web::*;

mod asset_loader;
#[cfg(not(target_family = "wasm"))]
#[path = "update_tick_handler_native.rs"]
mod update_tick_handler_native;

#[cfg(target_family = "wasm")]
#[path = "update_tick_handler_web.rs"]
mod update_tick_handler_web;

pub use asset_loader::*;
