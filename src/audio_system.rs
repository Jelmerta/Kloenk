use crate::resources::load_binary;
#[cfg(not(target_arch = "wasm32"))]
use rodio::{OutputStream, OutputStreamHandle, Sink};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
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
    sounds: HashMap<String, Sound>,
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub async fn new() -> Self {
        let sounds = Self::load_sounds().await;

        #[cfg(target_arch = "wasm32")]
        let audio_player = AudioPlayer::new();
        #[cfg(not(target_arch = "wasm32"))]
        let audio_player = AudioPlayer::new(sounds);

        AudioSystem {
            sounds,
            audio_player,
        }

        // user_has_gestured: bool
        // // TODO make sure run once
        // #[cfg(target_arch = "wasm32")]
        // {
        //     let audio_system_clone = engine.audio_system.clone();
        //     let has_gestured = engine.input_handler.user_has_gestured.clone();
        //     let audio_system_loaded = engine.audio_system.borrow().is_some();
        //     if !audio_system_loaded {
        //         spawn_local(async move {
        //             let mut audio_system = audio_system_clone.borrow_mut();
        //             if audio_system.is_none() && has_gestured {
        //                 *audio_system = Some(AudioSystem::new().await);
        //             }
        //         });
        //     }
        // }
        //
        // #[cfg(not(target_arch = "wasm32"))]
        // if engine.audio_system.is_none() {
        //     engine.audio_system = Some(pollster::block_on(AudioSystem::new()));
        // }
    }

    pub async fn start(&mut self) {
        self.audio_player.fill_buffers(&self.sounds).await;
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
struct AudioStream {
    sound_bytes: Sound,
    _stream: OutputStream, // Needs to be kept alive as long as handle lives to play audio
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}
//Impl audiostream: is_playing?

#[cfg(not(target_arch = "wasm32"))]
struct AudioPlayer {
    audio_streams: HashMap<String, AudioStream>,
}
#[cfg(not(target_arch = "wasm32"))]
impl AudioPlayer {
    pub fn new(sounds: HashMap<String, Sound>) -> Self {
        let mut audio_streams = HashMap::new();
        for (sound_name, sound) in sounds {
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let sink = None;
            let audio_stream = AudioStream {
                sound_bytes: sound,
                _stream,
                handle,
                sink,
            };
            audio_streams.insert(sound_name, audio_stream);
        }

        AudioPlayer { audio_streams }
    }

    pub fn is_playing(&self, sound: &str) -> bool {
        let audio_stream = self.audio_streams.get(sound);
        audio_stream.is_some()
            && audio_stream.unwrap().sink.is_some()
            && !audio_stream.unwrap().sink.as_ref().unwrap().empty()
    }

    pub fn play_sound(&mut self, sound: &str) {
        let audio_stream = self.audio_streams.get_mut(sound).unwrap();
        let audio_cursor = Cursor::new(audio_stream.sound_bytes.bytes.clone());
        let source = rodio::Decoder::new(audio_cursor).unwrap();
        let sink = Sink::try_new(&audio_stream.handle).unwrap();
        sink.append(source);
        audio_stream.sink = Some(sink);
    }
}

#[cfg(target_arch = "wasm32")]
struct AudioResource {
    audio_context: AudioContext,
    audio_buffer: AudioBuffer,
    is_playing: Rc<RefCell<bool>>,
}

// Was unable to get cpal/rodio working on wasm as no devices are returned from default device. Instead going for a web-sys implementation
#[cfg(target_arch = "wasm32")]
struct AudioPlayer {
    audio_resources: HashMap<String, AudioResource>,
}

#[cfg(target_arch = "wasm32")]
impl AudioPlayer {
    pub fn new() -> Self {
        AudioPlayer {
            audio_resources: HashMap::new(),
        }
    }

    fn is_playing(&self, sound: &str) -> bool {
        self.audio_resources
            .get(sound)
            .is_some_and(|sound| *sound.is_playing.borrow())
    }

    async fn fill_buffers(&mut self, sounds: &HashMap<String, Sound>) {
        for (sound_name, sound) in sounds {
            let audio_context = AudioContext::new().unwrap();

            let uint8_array = Uint8Array::new_with_length(sound.bytes.len() as u32);
            uint8_array.copy_from(&sound.bytes);
            let array_buffer = uint8_array.buffer();

            let promise = audio_context.decode_audio_data(&array_buffer).unwrap();
            let decoded_buffer = JsFuture::from(promise).await.unwrap();
            let audio_buffer = decoded_buffer.dyn_into::<AudioBuffer>().unwrap();

            let audio_resource = AudioResource {
                audio_context,
                audio_buffer,
                is_playing: Rc::new(RefCell::new(false)),
            };

            self.audio_resources
                .insert(sound_name.to_string(), audio_resource);
        }
    }

    pub fn play_sound(&mut self, sound: &str) {
        let audio_resource = self.audio_resources.get(sound).unwrap();
        let is_playing = audio_resource.is_playing.clone();
        let is_playing_set = audio_resource.is_playing.clone();
        let audio_buffer = audio_resource.audio_buffer.clone();
        let buffer_source = audio_resource.audio_context.create_buffer_source().unwrap();
        buffer_source.set_buffer(Some(&audio_buffer));
        let sound_name = sound.to_string();
        let remove_audio_closure = Closure::wrap(Box::new(move || {
            let mut mut_is_playing = is_playing.borrow_mut();
            *mut_is_playing = false;
        }) as Box<dyn FnMut()>);

        buffer_source
            .add_event_listener_with_callback(
                "ended",
                remove_audio_closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        buffer_source
            .connect_with_audio_node(
                &self
                    .audio_resources
                    .get(sound)
                    .unwrap()
                    .audio_context
                    .destination(),
            )
            .unwrap();

        buffer_source.start().unwrap();
        //let is_playing_clone = is_playing.clone();
        let mut mut_is_playing = is_playing_set.borrow_mut();
        *mut_is_playing = true;
        remove_audio_closure.forget();
    }
}
