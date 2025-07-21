use rodio::Sink;
use rodio::{OutputStream, OutputStreamBuilder};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Clone)]
pub struct Sound {
    bytes: Vec<u8>,
}

pub struct AudioSystem {
    audio_player: AudioPlayer,
}

impl AudioSystem {
    pub async fn new() -> Self {
        let sounds = Self::load_sounds();

        AudioSystem {
            audio_player: AudioPlayer::new(sounds.await),
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
            // #[cfg(not(target_arch = "wasm32"))]
            // bytes: pollster::block_on(hydro_utils::load_binary("bonk.ogg")).unwrap(), // Is it really just this easy? what about other file formats? Need a decoder? https://github.com/eshaz/wasm-audio-decoders/tree/master? wav(or pcm) is raw. probably want to use flac if we want lossless compression (smaller files without fidelity loss). other formats SHOULD require decoding. though i think mp3 just worked...

            // #[cfg(target_arch = "wasm32")]
            bytes: hydro_utils::load_binary("bonk.ogg").await.unwrap(), // Is it really just this easy? what about other file formats? Need a decoder? https://github.com/eshaz/wasm-audio-decoders/tree/master? wav(or pcm) is raw. probably want to use flac if we want lossless compression (smaller files without fidelity loss). other formats SHOULD require decoding. though i think mp3 just worked..
        };

        let mut sounds = HashMap::new();
        sounds.insert("bonk".to_string(), bonk_sound);
        sounds
    }
}

struct AudioResource {
    sound_bytes: Sound,
    sink: Option<Sink>,
}

struct AudioPlayer {
    audio_stream: OutputStream,
    audio_resources: HashMap<String, AudioResource>,
}
impl AudioPlayer {
    pub fn new(sounds: HashMap<String, Sound>) -> Self {
        let mut audio_resources = HashMap::new();
        let mut audio_stream = OutputStreamBuilder::open_default_stream().unwrap();
        audio_stream.log_on_drop(false);

        for (sound_name, sound) in sounds {
            let sink = None;
            let audio_resource = AudioResource {
                sound_bytes: sound,
                sink,
            };
            audio_resources.insert(sound_name, audio_resource);
        }

        AudioPlayer { audio_stream, audio_resources }
    }

    pub fn is_playing(&self, sound: &str) -> bool {
        let audio_resource = self.audio_resources.get(sound);
        audio_resource.is_some()
            && audio_resource.unwrap().sink.is_some()
            && !audio_resource.unwrap().sink.as_ref().unwrap().empty()
    }

    pub fn play_sound(&mut self, sound: &str) {
        let audio_resource = self.audio_resources.get_mut(sound).unwrap();
        let audio_cursor = Cursor::new(audio_resource.sound_bytes.bytes.clone());
        let sink = rodio::play(self.audio_stream.mixer(), audio_cursor);
        audio_resource.sink = Some(sink.unwrap());
    }
}