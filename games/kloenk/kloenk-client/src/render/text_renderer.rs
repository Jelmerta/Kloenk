use crate::state::ui_state::UIElement;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use hydrox::load_binary;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};
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
    swash_cache: SwashCache, // TODO when to empty? does this grow?
    viewport: Viewport,
    atlas: TextAtlas,
    queue: Vec<TextContext>,
}

impl TextWriter {
    pub async fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // TODO load as resource later, not blocking the renderer?
        let font_data = load_binary("PlaywriteNL-Minimal.ttf").await.unwrap(); // TODO retry?

        let mut fontdb = fontdb::Database::new();
        fontdb.load_font_data(font_data);
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_owned(), fontdb);

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

    pub fn reset_for_update(&mut self) {
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
            Some(window.inner_size().width as f32), // TODO does not change on resize?
            Some(window.inner_size().height as f32),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue.push(TextContext {
            buffer,
            ui_element: *ui_element,
            color: *color,
        });
    }

    // TODO only call when UI changes?
    pub fn prepare(&mut self, device: &Device, queue: &Queue, window: &Arc<Window>) {
        // TODO only update viewport on resize?
        self.viewport.update(
            queue,
            Resolution {
                width: window.inner_size().width,
                height: window.inner_size().height,
            },
        );

        // TODO seems pretty heavyweight method, probably check. Even if only one text area changes, we need to call it with all? Maybe just create new atlas/queue for the updated parts?
        // TODO wonder if we don't want our own text rendering. Just disable more expensive things like kerning/shaping
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
                    .collect::<Vec<_>>(),
                &mut self.swash_cache,
            )
            .unwrap(); // TODO handle error
    }

    // TODO prepare earlier? also if this helps note that this is an expensive method to call apparently
    // glyphon even uses its own pipeline, probably own shader. sounds pretty inefficient. maybe it's fine since we can reuse render pass at least?
    pub fn write_text_buffer(&mut self, render_pass: &mut RenderPass) {
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}
