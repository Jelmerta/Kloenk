use crate::resources::load_binary;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioBuffer;

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

    pub fn play_sound(&mut self, sound: &str) {
        // if self.audio_player.is_playing() {
        //     return;
        // }
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
    sink: Option<Sink>,
}
#[cfg(not(target_arch = "wasm32"))]
impl AudioPlayer {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        AudioPlayer {
            _stream,
            handle,
            sink: None,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.sink.is_some() && !self.sink.as_ref().unwrap().empty()
    }

    pub fn play_sound(&mut self, sound: &Sound) {
        let audio_cursor = Cursor::new(sound.bytes.clone());
        let source = rodio::Decoder::new(audio_cursor).unwrap();
        // self.handle.play_raw(source.convert_samples()).unwrap();
        let sink = Sink::try_new(&self.handle).unwrap();
        sink.append(source);
        self.sink = Some(sink);
    }
}

// Was unable to get cpal/rodio working on wasm as no devices are returned from default device. Instead going for a web-sys implementation
#[cfg(target_arch = "wasm32")]
pub struct AudioPlayer {
    audio_context: web_sys::AudioContext,
    audio_buffer: AudioBuffer,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayer {
    pub fn new(sound: &Sound) -> Self {
        let audio_context = web_sys::AudioContext::new().unwrap();
        let audio_buffer;
        let promise = audio_context.decode_audio_data(sound.bytes.as_ref());
        audio_buffer = promise.then(|buffer| {
            audio_buffer = buffer;
        });

        AudioPlayer {
            audio_context,
            audio_buffer,
        }
    }

    pub fn play_sound(&self, _sound: &Sound) {
        let buffer_source = self.audio_context.create_buffer_source().unwrap();
        buffer_source.set_buffer(Some(&self.audio_buffer));
        buffer_source.connect(self.audio_context.destination());
        buffer_source.start().unwrap();
    }
}
