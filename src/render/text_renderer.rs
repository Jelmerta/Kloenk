use crate::resources;
use crate::state::ui_state::Rect;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use std::collections::HashMap;
use wgpu::{Adapter, Device, Queue, Surface};

pub struct TextWriter {
    text_renderer: TextRenderer,
    pub font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    pub text_buffers: HashMap<String, Buffer>,
    // selected_text_buffer: Buffer,
    // pub action_text_buffer: Buffer,
}

#[allow(clippy::cast_possible_truncation)]
impl TextWriter {
    pub async fn new(
        device: &Device,
        queue: &Queue,
        surface: &Surface<'_>,
        adapter: &Adapter,
        window_width: f32,
        window_height: f32,
    ) -> Self {
        let font_data = resources::load_binary("PlaywriteNL-Regular.ttf")
            .await
            .unwrap();

        let mut fontdb = fontdb::Database::new();
        fontdb.load_font_data(font_data);
        let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), fontdb);

        let caps = surface.get_capabilities(adapter);
        let surface_format = caps // I see tutorial using wgpu::TextureFormat::Bgra8UnormSrgb
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(caps.formats[0]);
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let mut selected_text_buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
        selected_text_buffer.set_size(&mut font_system, Some(window_width), Some(window_height));
        selected_text_buffer.shape_until_scroll(&mut font_system, false);

        let mut action_text_buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
        action_text_buffer.set_size(&mut font_system, Some(window_width), Some(window_height));
        action_text_buffer.shape_until_scroll(&mut font_system, false);

        // TODO have not found a way yet to handle dynamic buffers correctly (so we have to pre-create buffers...)
        let mut text_buffers = HashMap::new();
        text_buffers.insert("selected_text".to_string(), selected_text_buffer);
        text_buffers.insert("action_text".to_string(), action_text_buffer);

        TextWriter {
            text_renderer,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_buffers,
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_width: u32,
        screen_height: u32,
        rect: Rect,
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: screen_width,
                height: screen_height,
            },
        );

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    // buffer: &self.action_text_buffer,
                    buffer: self.text_buffers.get("action_text").unwrap(),
                    left: rect.top_left.x,
                    top: rect.top_left.y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: rect.top_left.x as i32 - 10, // Adding 10 for some padding so text is fully shown
                        top: rect.top_left.y as i32 - 10,
                        right: rect.bottom_right.x as i32,
                        bottom: rect.bottom_right.y as i32,
                    },
                    default_color: Color::rgb(255, 255, 0),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        // self.action_text_buffer
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn write_text_buffer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        text: &str,
    ) {
        let buffer = self.text_buffers.get_mut("action_text").unwrap();

        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Advanced,
        );

        // / self.action_text_buffer.set_text(
        //     &mut self.font_system,
        //     text,
        //     Attrs::new().family(Family::Name("Playwrite NL")),
        //     Shaping::Advanced,
        // );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .unwrap();
        }
    }
}
