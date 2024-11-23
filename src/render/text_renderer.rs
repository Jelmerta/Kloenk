use crate::resources;
use crate::state::ui_state::Rect;
use cgmath::Vector3;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use itertools::Itertools;
use wgpu::{Adapter, CommandEncoder, Device, Queue, Surface, TextureView};

struct TextContext {
    buffer: Buffer,
    rect: Rect,
    color: Vector3<f32>, // TODO f32 is meh for this...
}

impl TextContext {
    fn to_text_area(&self) -> TextArea {
        TextArea {
            buffer: &self.buffer,
            left: self.rect.top_left.x,
            top: self.rect.top_left.y,
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgb(self.color.x as u8, self.color.y as u8, self.color.z as u8),
            custom_glyphs: &[],
        }
    }
}

pub struct TextWriter {
    text_renderer: TextRenderer,
    pub font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    queue: Vec<TextContext>,
}

#[allow(clippy::cast_possible_truncation)]
impl TextWriter {
    pub async fn new(
        device: &Device,
        queue: &Queue,
        surface: &Surface<'_>,
        adapter: &Adapter,
    ) -> Self {
        let font_data = resources::load_binary("PlaywriteNL-Regular.ttf")
            .await
            .unwrap();

        let mut fontdb = fontdb::Database::new();
        fontdb.load_font_data(font_data);
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), fontdb);

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

        TextWriter {
            text_renderer,
            font_system,
            swash_cache,
            viewport,
            atlas,
            queue: Vec::new(),
        }
    }

    pub fn reset_for_frame(&mut self) {
        self.atlas.trim();
        self.queue.clear();
    }

    pub fn add(&mut self, screen_width: u32, screen_height: u32, rect: Rect, text: &str) {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
        buffer.set_size(
            &mut self.font_system,
            Some(screen_width as f32),
            Some(screen_height as f32),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue.push(TextContext {
            buffer,
            rect,
            color: Vector3::new(255.0, 255.0, 0.0),
        });
    }

    pub fn write(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_width: u32,
        screen_height: u32,
    ) {
        self.prepare(device, queue, screen_width, screen_height);
        self.write_text_buffer(encoder, view);
    }

    fn prepare(&mut self, device: &Device, queue: &Queue, screen_width: u32, screen_height: u32) {
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
                self.queue
                    .iter()
                    .map(|text_context| text_context.to_text_area())
                    .collect_vec(),
                &mut self.swash_cache,
            )
            .unwrap();
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_text_buffer(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
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
            drop(pass);
        }
    }
}
