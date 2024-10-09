use crate::resources::load_binary;
use std::io::Cursor;

pub struct AudioPlayer {
    pub tmp: String,
    // sounds: HashMap<String, Sound>,
}

impl AudioPlayer {
    // pub async fn new() -> Self {
    pub fn new() -> Self {
        AudioPlayer {
            // sounds: HashMap::new(),
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
    pub fn play_audio() {
        std::thread::spawn(move || {
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();

            let sound_bytes = pollster::block_on(load_binary("bonk.mp3")).unwrap();
            let audio_cursor = Cursor::new(sound_bytes);
            sink.append(rodio::Decoder::new(audio_cursor).unwrap());

            sink.sleep_until_end();
        });
    }
}

// struct Sound {
// bytes: Vec<u8>,
// }

//
