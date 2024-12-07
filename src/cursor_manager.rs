use crate::resources::load_binary;
use winit::window::{CustomCursor, CustomCursorSource};

pub struct CursorManager {}
impl CursorManager {
    pub async fn load_cursor_future() -> Vec<u8> {
        load_binary("cursor.png").await.unwrap()
    }

    pub fn load_cursor(cursor_binary: Vec<u8>) -> CustomCursorSource {
        let cursor_rgba = image::load_from_memory(&cursor_binary)
            .unwrap()
            .to_rgba8()
            .into_raw();
        CustomCursor::from_rgba(cursor_rgba, 122, 120, 7, 7).unwrap()
    }
}
