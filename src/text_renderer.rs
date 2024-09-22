use std::env;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Resolution,
};
use wgpu::{Device, Queue, Surface, Adapter};
use anyhow::anyhow;

pub struct TextWriter {
    text_renderer: TextRenderer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_buffer: Buffer,
}

impl TextWriter {
    pub fn new(device: &Device, queue: &Queue, surface: &Surface, adapter: &Adapter) -> Self {
        let mut font_system = FontSystem::new();


        let mut out_dir = env::var("OUT_DIR").unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            out_dir = format!("{}/", out_dir);
        }
         font_system
             .db_mut()
            .load_font_file(format!("{}resources/PlaywriteNL-Regular.ttf", out_dir))
             .map_err(|e| anyhow!("Failed to copy items: {:?}", e))
            .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps // I see tutorial using wgpu::TextureFormat::Bgra8UnormSrgb
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);                                       // format potentially on surface
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (800.0 * 1.0) as f32;
        let physical_height = (600.0 * 1.0) as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(&mut font_system, "Kijk hoe fancy deze tekst :O!\nZo gaaf!!日本語も大丈夫そう。。。", Attrs::new().family(Family::Name("Playwrite NL")), Shaping::Advanced);
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

    pub fn write(&mut self, device: &Device, queue: &Queue, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
            self.viewport.update(
                    &queue,
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
                    left: 10.0,
                    top: 400.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 800,
                        bottom: 600,
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
                    view: &view,
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
