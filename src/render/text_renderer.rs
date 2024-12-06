use crate::resources;
use crate::state::ui_state::Rect;
use cgmath::num_traits::Pow;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use itertools::Itertools;
use std::ops::Add;
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};
use winit::window::Window;

const DEFAULT_FONT_SIZE: f32 = 24.0;
const DEFAULT_FONT_HEIGHT: f32 = 1080.0; // Using a default resolution to scale by, as dpi/pixelratio is independent of window size
const DEFAULT_FONT_WIDTH: f32 = 1920.0; // Using a default resolution to scale by, as dpi/pixelratio is independent of window size

struct TextContext {
    buffer: Buffer,
    rect: Rect,
    color: [f32; 3],
}

impl TextContext {
    fn to_text_area(&self) -> TextArea {
        // Hm... Kind of implicit conversion logic hidden deep...
        let text_color = [
            (self.color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        TextArea {
            buffer: &self.buffer,
            left: self.rect.top_left.x.add(5.0), // Give some space on top and left for text
            top: self.rect.top_left.y.add(5.0),
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgb(text_color[0], text_color[1], text_color[2]),
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
    pub async fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let font_data = resources::load_binary("PlaywriteNL-Regular.ttf")
            .await
            .unwrap();

        let mut fontdb = fontdb::Database::new();
        fontdb.load_font_data(font_data);
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), fontdb);

        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, config.format.add_srgb_suffix());
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

    pub fn add(&mut self, window: &Arc<Window>, rect: &Rect, text: &str, color: &[f32; 3]) {
        let rect_scaled = rect.scale(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        );
        let default_diagonal_distance: f32 =
            (DEFAULT_FONT_WIDTH.pow(2) as f32 + DEFAULT_FONT_HEIGHT.pow(2) as f32).sqrt();
        let current_diagonal_distance: f32 = (window.inner_size().width.pow(2) as f32
            + window.inner_size().height.pow(2) as f32)
            .sqrt();
        let font_size = (current_diagonal_distance / default_diagonal_distance) * DEFAULT_FONT_SIZE;
        log::warn!("hi");
        log::warn!("{}", default_diagonal_distance);
        log::warn!("{}", current_diagonal_distance);
        log::warn!("{}", font_size);
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, font_size * 2.0),
        );
        buffer.set_size(
            &mut self.font_system,
            Some(window.inner_size().width as f32),
            Some(window.inner_size().height as f32),
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
            rect: rect_scaled,
            color: *color,
        });
    }

    pub fn write(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        window: &Arc<Window>,
    ) {
        self.prepare(device, queue, window);
        self.write_text_buffer(encoder, view);
    }

    fn prepare(&mut self, device: &Device, queue: &Queue, window: &Arc<Window>) {
        self.viewport.update(
            queue,
            Resolution {
                width: window.inner_size().width,
                height: window.inner_size().height,
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
