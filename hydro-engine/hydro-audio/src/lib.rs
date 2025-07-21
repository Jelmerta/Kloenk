#[cfg(target_arch = "wasm32")]
// #[path = "audio_player_web.rs"]
#[path = "audio_player_native.rs"]
mod audio_player;

#[cfg(not(target_arch = "wasm32"))]
#[path = "audio_player_native.rs"]
mod audio_player;

pub use audio_player::*;
