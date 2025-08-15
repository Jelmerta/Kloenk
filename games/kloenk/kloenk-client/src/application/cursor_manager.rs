use winit::window::{CustomCursor, CustomCursorSource};

pub struct CursorManager {}
impl CursorManager {
    pub fn load_cursor(cursor_binary: Vec<u8>) -> CustomCursorSource {
        // let cursor_rgba = image::load_from_memory(&cursor_binary)
        //     .unwrap()
        //     .to_rgba8()
        //     .into_raw();
        CustomCursor::from_rgba(cursor_binary, 61, 60, 3, 3).unwrap()
    }
}
