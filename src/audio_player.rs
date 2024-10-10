// use wasm_thread as thread;
#[cfg(not(target_arch = "wasm32"))]
use rodio::Source;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlAudioElement;

pub struct AudioPlayer {
    pub tmp: String,
    // stream: OutputStream,
    // handle: OutputStreamHandle,
    // sink: Sink,
    // sounds: HashMap<String, Sound>,
}

impl AudioPlayer {
    // pub async fn new() -> Self {
    pub fn new() -> Self {
        // let (stream, handle) = OutputStream::try_default().unwrap();
        // let (stream, handle) = OutputStream::try_from_device().unwrap();
        // let sink = Sink::try_new(&handle).unwrap();

        AudioPlayer {
            // sounds: HashMap::new(),
            // stream,
            // handle,
            // sink,
            tmp: String::new(),
        }

        // Probably split this up such that audio player does not have data
        // or maybe run on separate thread
        // audio_player.load_sounds().await;
        // audio_player.load_sounds();

        // audio_player
    }

    // async fn load_sounds(&mut self) {
    // fn load_sounds(&mut self) {
    // fn load_sounds() {

    //
    // let bonk_sound = Sound {
    // bytes: load_binary("bonk.mp3").await.unwrap(),
    // };

    // self.sounds.insert("bonk".to_string(), bonk_sound);
    // }

    //, sound: &str
    pub fn play_audio(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();

            let sound_bytes = pollster::block_on(load_binary("bonk.wav")).unwrap();
            let audio_cursor = Cursor::new(sound_bytes);
            let source = rodio::Decoder::new(audio_cursor).unwrap();
            handle.play_raw(source.convert_samples()).unwrap();
            // sink.append(source);
            // sink.sleep_until_end();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // let bonk_binary = pollster::block_on(load_binary("bonk.mp3")).unwrap();
            // let audio_context = AudioContext::new();
            let audio_element = HtmlAudioElement::new_with_src("resources/bonk.mp3").unwrap();
            audio_element.set_autoplay(true);
            audio_element.play().unwrap();
        }
    }
}

// struct Sound {
// bytes: Vec<u8>,
// }

//
