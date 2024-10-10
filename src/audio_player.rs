use crate::resources::load_binary;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;
// use wasm_thread as thread;

pub struct AudioPlayer {
    pub tmp: String,
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    // sounds: HashMap<String, Sound>,
}

impl AudioPlayer {
    // pub async fn new() -> Self {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();

        AudioPlayer {
            // sounds: HashMap::new(),
            stream,
            sink,
            handle,
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
        // std::thread::spawn(move || {
        // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        // let sink = rodio::Sink::try_new(&handle).unwrap();

        let sound_bytes = pollster::block_on(load_binary("bonk.wav")).unwrap();
        let audio_cursor = Cursor::new(sound_bytes);
        let source = rodio::Decoder::new(audio_cursor).unwrap();

        self.handle.play_raw(source.convert_samples()).unwrap();
        // self.sink.append(source);

        // self.sink.sleep_until_end();
        // self.sink.;
        // thread::sleep(Duration::from_millis(3000));
        // self.sink.detach();
        // });
    }
}

// struct Sound {
// bytes: Vec<u8>,
// }

//
