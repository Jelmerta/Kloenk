mod cursor_manager;

#[cfg(target_arch = "wasm32")]
#[path = "application_web.rs"]
mod application;

#[cfg(not(target_arch = "wasm32"))]
#[path = "application_native.rs"]
mod application;
mod asset_loader;

pub use application::*;
pub use asset_loader::*;
