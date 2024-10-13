use crate::resources::load_binary;
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::js_sys::Uint8Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioBuffer;
#[cfg(target_arch = "wasm32")]
use web_sys::AudioContext;

struct Sound {
    bytes: Vec<u8>,
}

pub struct AudioSystem {
    sounds_in_binary: HashMap<String, Sound>,
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub async fn new() -> Self {
        let sounds_in_binary = Self::load_sounds().await;
        let audio_player = AudioPlayer::new(&sounds_in_binary).await;

        AudioSystem {
            sounds_in_binary,
            audio_player,
        }
    }

    pub fn play_sound(&mut self, sound: &str) {
        if self.audio_player.is_playing(sound) {
            return;
        }
        self.audio_player.play_sound(sound);
    }

    async fn load_sounds() -> HashMap<String, Sound> {
        let bonk_sound = Sound {
            bytes: load_binary("bonk.wav").await.unwrap(),
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
    audio_buffers: Rc<RefCell<HashMap<String, AudioBuffer>>>,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayer {
    pub async fn new(sounds: &HashMap<String, Sound>) -> Self {
        let audio_context = AudioContext::new().unwrap(); // TODO Should load on user gesture instead of immediately https://goo.gl/7K7WLu
        let audio_buffers = Rc::new(RefCell::new(
            Self::load_buffers(&audio_context, sounds).await,
        ));

        AudioPlayer {
            audio_context,
            audio_buffers,
        }
    }

    fn is_playing(&self, sound: &str) -> bool {
        self.audio_buffers.borrow().get(sound).is_some()
    }

    async fn load_buffers(
        audio_context: &AudioContext,
        sounds: &HashMap<String, Sound>,
    ) -> HashMap<String, AudioBuffer> {
        let mut audio_buffers = HashMap::new();
        for (sound_name, sound) in sounds {
            let uint8_array = Uint8Array::new_with_length(sound.bytes.len() as u32);
            uint8_array.copy_from(&sound.bytes);
            let array_buffer = uint8_array.buffer();

            let promise = audio_context.decode_audio_data(&array_buffer).unwrap();
            let decoded_buffer = JsFuture::from(promise).await.unwrap();
            let audio_buffer = decoded_buffer.dyn_into::<AudioBuffer>().unwrap();

            audio_buffers.insert(sound_name.to_string(), audio_buffer);
        }
        audio_buffers
    }

    pub fn play_sound(&mut self, sound: &str) {
        let audio_buffers = self.audio_buffers.clone();
        let audio_buffer = audio_buffers.borrow().get(sound).unwrap().clone();
        let buffer_source = self.audio_context.create_buffer_source().unwrap();
        buffer_source.set_buffer(Some(&audio_buffer));
        let sound_name = sound.to_string();
        let remove_audio_closure = Closure::wrap(Box::new(move || {
            audio_buffers.borrow_mut().remove(&sound_name);
        }) as Box<dyn FnMut()>);

        buffer_source
            .add_event_listener_with_callback(
                "ended",
                remove_audio_closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        buffer_source
            .connect_with_audio_node(&self.audio_context.destination())
            .unwrap();

        buffer_source.start().unwrap();
    }
}

// Closure::wrap(Box::new(move || &self.audio_buffers.remove(sound)))
//     .into_js_result()
//     .unwrap()
//     .unchecked_into::<Function>(),
