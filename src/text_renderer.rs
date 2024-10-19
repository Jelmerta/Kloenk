use crate::gui::UIState;
use crate::resources;
use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{Adapter, Device, Queue, Surface};

pub struct TextWriter {
    text_renderer: TextRenderer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    selected_text_buffer: Buffer,
    action_text_buffer: Buffer,
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
        let mut viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let physical_width = (800.0 * 1.0) as f32;
        let physical_height = (600.0 * 1.0) as f32;

        let mut selected_text_buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));

        selected_text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        selected_text_buffer.shape_until_scroll(&mut font_system, false);

        let mut action_text_buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));

        action_text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        action_text_buffer.shape_until_scroll(&mut font_system, false);

        TextWriter {
            text_renderer,
            font_system,
            swash_cache,
            viewport,
            atlas,
            selected_text_buffer,
            action_text_buffer,
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, ui_state: &UIState) {
        self.viewport.update(
            queue,
            Resolution {
                width: ui_state.window_size.width,
                height: ui_state.window_size.height,
            },
        );

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [
                    TextArea {
                        buffer: &self.selected_text_buffer,
                        left: ui_state.window_size.width as f32
                            * ui_state.selected_text.position_top_left.x,
                        top: ui_state.window_size.height as f32
                            * ui_state.selected_text.position_top_left.y,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: (ui_state.window_size.width as f32
                                * ui_state.selected_text.position_top_left.x)
                                as i32
                                - 10, // Adding 10 for some padding so text is fully shown
                            top: (ui_state.window_size.height as f32
                                * ui_state.selected_text.position_top_left.y)
                                as i32
                                - 10,
                            right: (ui_state.window_size.width as f32
                                * ui_state.selected_text.position_bottom_right.x)
                                as i32,
                            bottom: (ui_state.window_size.height as f32
                                * ui_state.selected_text.position_bottom_right.y)
                                as i32,
                        },
                        default_color: Color::rgb(255, 255, 0),
                        custom_glyphs: &[],
                    },
                    TextArea {
                        buffer: &self.action_text_buffer,
                        left: ui_state.window_size.width as f32
                            * ui_state.action_text.position_top_left.x,
                        top: ui_state.window_size.height as f32
                            * ui_state.action_text.position_top_left.y,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: (ui_state.window_size.width as f32
                                * ui_state.action_text.position_top_left.x)
                                as i32
                                - 10, // Adding 10 for some padding so text is fully shown
                            top: (ui_state.window_size.height as f32
                                * ui_state.action_text.position_top_left.y)
                                as i32
                                - 10,
                            right: (ui_state.window_size.width as f32
                                * ui_state.action_text.position_bottom_right.x)
                                as i32,
                            bottom: (ui_state.window_size.height as f32
                                * ui_state.action_text.position_bottom_right.y)
                                as i32,
                        },
                        default_color: Color::rgb(255, 255, 0),
                        custom_glyphs: &[],
                    },
                ],
                &mut self.swash_cache,
            )
            .unwrap();
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn write_selected_text_buffer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        text: &str,
    ) {
        self.selected_text_buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Advanced,
        );

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

    #[allow(clippy::cast_possible_truncation)]
    pub fn write_action_text_buffer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        text: &str,
    ) {
        self.action_text_buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Name("Playwrite NL")),
            Shaping::Advanced,
        );

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
