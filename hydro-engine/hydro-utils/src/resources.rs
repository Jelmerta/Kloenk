#[cfg(target_arch = "wasm32")]
#[path = "resources_web.rs"]
mod resources;

#[cfg(not(target_arch = "wasm32"))]
#[path = "resources_native.rs"]
mod resources;

pub use resources::*;
