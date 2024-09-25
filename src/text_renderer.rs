use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Resolution,
};
use wgpu::{Device, Queue, Surface, Adapter};
use crate::gui::UIState;
use crate::resources;

pub struct TextWriter {
    text_renderer: TextRenderer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_buffer: Buffer,
}

impl TextWriter {
    pub async fn new(device: &Device, queue: &Queue, surface: &Surface<'_>, adapter: &Adapter) -> Self {
        let mut font_system = FontSystem::new();

        let font_data = resources::load_binary("PlaywriteNL-Regular.ttf").await.unwrap();

        font_system
            .db_mut()
            .load_font_data(font_data.to_vec());

        let caps = surface.get_capabilities(adapter);
        let surface_format = caps // I see tutorial using wgpu::TextureFormat::Bgra8UnormSrgb
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 20.0));

        let physical_width = (800.0 * 1.0) as f32;
        let physical_height = (600.0 * 1.0) as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        TextWriter {
            text_renderer,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_buffer,
        }
    }

    pub fn write(&mut self, device: &Device, queue: &Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, ui_state: &UIState) {
        self.text_buffer.set_text(&mut self.font_system, ui_state.text.as_str(), Attrs::new().family(Family::Name("Playwrite NL")), Shaping::Advanced);

        self.viewport.update(
            queue,
            Resolution {
                width: 800,
                height: 600,
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
                    buffer: &self.text_buffer,
                    left: 800.0 * ui_state.text_position_x,
                    top: 600.0 * ui_state.text_position_y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: (800.0 * ui_state.text_position_x) as i32 - 10, // Adding 10 for some padding so text is fully shown
                        top: (600.0 * ui_state.text_position_y) as i32 - 10,
                        right: (800.0 * ui_state.text_position_x + 800.0 * ui_state.text_width) as i32,
                        bottom: (600.0 * ui_state.text_position_y + 600.0 * ui_state.text_height) as i32,
                    },
                    default_color: Color::rgb(255, 255, 0),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

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
