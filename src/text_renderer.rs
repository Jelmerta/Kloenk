use glyphon::cosmic_text::fontdb;
use glyphon::fontdb::Source;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Resolution,
};
use wgpu::{Device, Queue, Surface, Adapter};
use std::sync::Arc;

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
        let mut font_db = fontdb::Database::new();
    let font_data = include_bytes!("../resources/Lohengrinn.ttf");
        font_db.load_font_data(font_data.to_vec());
        let fonts = vec![Source::Binary(Arc::new(font_data.to_vec()))];
        let mut font_system = FontSystem::new_with_fonts(fonts);
        // TODO HOW DO WE not use default system
        // HOW DO WE USE CUSTOM TTF
        // font_system.db_mut().set_serif_family("Times New Roman");
        font_system.db_mut().set_sans_serif_family("Lohengrinn");
        // font_system.db_mut().set_cursive_family("Comic Sans MS");
        // font_system.db_mut().set_monospace_family("Courier New");
        // font_system.db_mut().set_fantasy_family("Impact");
        // font_system
            // .db_mut()
            // .load_font_file("../resources/Lohengrinn.ttf")
            // .map_err(|e| anyhow!("Failed to copy items: {:?}", e))
            // .unwrap();
        // let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // TODO different swapchain
        // let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // TODO different swapchain
                             
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);                                       // format potentially on surface
        // device.swap
        // wgpu::Swap
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (800 as f64 * 1.0) as f32;
        let physical_height = (600 as f64 * 1.0) as f32;

        // text_buffer.

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(&mut font_system, "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
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

    pub fn write(&mut self, device: &Device, queue: &Queue, surface: &Surface) {
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
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 800,
                        bottom: 600,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        let frame = surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // pass.set_pipeline(pipeline)

            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .unwrap();
        }
    }
}
