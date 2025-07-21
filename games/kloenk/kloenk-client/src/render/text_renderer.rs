use crate::state::ui_state::UIElement;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use hydrox::load_binary;
use itertools::Itertools;
use std::sync::Arc;
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};
use winit::window::Window;

const DEFAULT_FONT_SIZE: f32 = 24.0;
const DEFAULT_FONT_HEIGHT: f32 = 1080.0; // Using a default resolution to scale by, as dpi/pixelratio is independent of window size

struct TextContext {
    buffer: Buffer,
    ui_element: UIElement,
    color: [f32; 3],
}

impl TextContext {
    fn to_text_area(&self, window: &Arc<Window>) -> TextArea<'_> {
        // Hm... Kind of implicit conversion logic hidden deep...
        let text_color = [
            (self.color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        // Left side is adjusted by first undoing window ratio and then scaling by 16:9
        let left = self.ui_element.ui_coordinate_origin.x
            - (self.ui_element.width() / 2.0)
            * (window.inner_size().height as f32 / window.inner_size().width as f32)
            * (16.0 / 9.0);
        let top = self.ui_element.top();

        TextArea {
            buffer: &self.buffer,
            top: top * window.inner_size().height as f32,
            left: left * window.inner_size().width as f32,
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
        let woff2_data = load_binary("PlaywriteNL-Regular.woff2")
            .await
            .unwrap();

        // let mut font_data = Vec::new();
        // BrotliDecompress(
        //     &mut &woff2[..],
        //     &mut font_data,
        // ).expect("Failed to decompress WOFF2");

        // ttf_parser::
        // let font_data = woff::version2::decompress(woff2.into()).unwrap();

        let font_data = woff2_patched::convert_woff2_to_ttf(&mut std::io::Cursor::new(woff2_data)).unwrap();
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

    pub fn add(
        &mut self,
        window: &Arc<Window>,
        ui_element: &UIElement,
        text: &str,
        color: &[f32; 3],
    ) {
        let font_size = window.inner_size().height as f32 / DEFAULT_FONT_HEIGHT * DEFAULT_FONT_SIZE;
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
            &Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue.push(TextContext {
            buffer,
            ui_element: *ui_element,
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
                    .map(|text_context| text_context.to_text_area(window))
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
