use crate::resources::load_binary;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioBuffer;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioContext;

struct Sound {
    bytes: Vec<u8>,
}

pub struct AudioSystem {
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub fn new() -> Self {
        let sounds = Self::load_sounds();
        AudioSystem {
            audio_player: AudioPlayer::new(sounds),
        }
    }

    pub fn play_sound(&mut self, sound: &str) {
        // if self.audio_player.is_playing() {
        //     return;
        // }
        // let sound = self.sounds.get(sound).unwrap();
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
    sounds: HashMap<String, Sound>,
    _stream: OutputStream, // Needs to be kept alive as long as handle lives to play audio
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}
#[cfg(not(target_arch = "wasm32"))]
impl AudioPlayer {
    pub fn new(sounds: HashMap<String, Sound>) -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        AudioPlayer {
            sounds,
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
    audio_context: AudioContext,
    audio_buffers: HashMap<String, AudioBuffer>,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayer {
    pub fn new(sounds: HashMap<String, Sound>) -> Self {
        let audio_context = AudioContext::new().unwrap();
        let audio_buffers = Self::load_buffers(&audio_context, sounds);

        AudioPlayer {
            audio_context,
            audio_buffers,
        }
    }

    fn load_buffers(
        audio_context: &AudioContext,
        sounds: HashMap<String, Sound>,
    ) -> HashMap<String, AudioBuffer> {
        let mut audio_buffers = HashMap::new();
        for (sound_name, sound) in sounds {
            let uint8_array = Uint8Array::new_with_length(sound.bytes.len() as u32);
            uint8_array.copy_from(&sound.bytes);
            let array_buffer = uint8_array.buffer();

            let promise = audio_context.decode_audio_data(&array_buffer).unwrap();
            let decoded_buffer = pollster::block_on(JsFuture::from(promise)).unwrap();
            let audio_buffer = decoded_buffer.dyn_into::<AudioBuffer>().unwrap();

            // audio_buffer = promise.then(|buffer| {
            //     audio_buffer = buffer;
            // });
            audio_buffers.insert(sound_name, audio_buffer);
        }
        audio_buffers
    }

    pub fn play_sound(&self, sound: &str) {
        let audio_buffer = self.audio_buffers.get(sound).unwrap();
        let buffer_source = self.audio_context.create_buffer_source().unwrap();
        buffer_source.set_buffer(Some(audio_buffer));
        buffer_source
            .connect_with_audio_node(&self.audio_context.destination())
            .unwrap();
        buffer_source.start().unwrap();
    }
}
