// use wasm_thread as thread;
#[cfg(not(target_arch = "wasm32"))]
use crate::resources::load_binary;
#[cfg(not(target_arch = "wasm32"))]
use rodio::Source;
use rodio::{OutputStream, OutputStreamHandle};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioContext;

struct Sound {
    bytes: Vec<u8>,
}

pub struct AudioSystem {
    sounds: HashMap<String, Sound>,
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub fn new() -> Self {
        AudioSystem {
            sounds: Self::load_sounds(),
            audio_player: AudioPlayer::new(),
        }
    }

    pub fn play_sound(&self, sound: &str) {
        let sound = self.sounds.get(sound).unwrap();
        self.audio_player.play_sound(sound);
    }

    fn load_sounds() -> HashMap<String, Sound> {
        let bonk_sound = Sound {
            bytes: pollster::block_on(load_binary("bonk.wav")).unwrap(),
        };

        let mut sounds = HashMap::new();
        sounds.insert("bonk".to_string(), bonk_sound);
        sounds
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct AudioPlayer {
    _stream: OutputStream, // Needs to be kept alive as long as handle lives to play audio
    handle: OutputStreamHandle,
}
#[cfg(not(target_arch = "wasm32"))]
impl AudioPlayer {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        AudioPlayer { _stream, handle }
    }
    pub fn play_sound(&self, sound: &Sound) {
        let audio_cursor = Cursor::new(sound.bytes.clone());
        let source = rodio::Decoder::new(audio_cursor).unwrap();
        self.handle.play_raw(source.convert_samples()).unwrap();
    }
}

// Was unable to get cpal/rodio working on wasm as no devices are returned from default device. Instead going for a web-sys implementation
#[cfg(target_arch = "wasm32")]
pub struct AudioPlayer {
    audio_context: web_sys::AudioContext,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayer {
    pub fn play_audio(&self) {
        let audio_element = HtmlAudioElement::new_with_src("resources/bonk.wav").unwrap();

        audio_element.set_autoplay(true);
        audio_element.play().unwrap();
    }
}
