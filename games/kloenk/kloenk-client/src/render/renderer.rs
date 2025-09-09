use crate::application::{FontAsset, ImageAsset};
use crate::render::camera::Camera;
use crate::render::camera_manager::CameraManager;
use crate::render::color_manager::ColorManager;
use crate::render::instance::InstanceRaw;
use crate::render::material_manager::TextureManager;
use crate::render::model::ColorDefinition;
use crate::render::model_manager::ModelManager;
use crate::render::primitive_vertices_manager::{PrimitiveVertices, PrimitiveVerticesManager};
use crate::render::render_context_manager::RenderContextManager;
use crate::render::text_renderer::TextWriter;
use crate::render::texture;
use crate::state::components::Scale;
use crate::state::game_state::GameState;
use crate::state::ui_state::{RenderCommand, UIElement, UIState};
use crate::state::update_state::UpdateState;
use cgmath::{prelude::*, Point3, Vector3};
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::CompositeAlphaMode::Auto;
use wgpu::PresentMode::{AutoVsync, Mailbox};
use wgpu::{
    Adapter, Buffer, CommandEncoder, Device, Features, InstanceFlags, MemoryHints, Queue,
    RenderPass, SurfaceConfiguration, TextureView, Trace,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    pub model_manager: ModelManager,
    primitive_vertices_manager: PrimitiveVerticesManager,
    color_manager: ColorManager,
    texture_manager: TextureManager,
    camera_manager: CameraManager,
    render_context_manager: RenderContextManager,

    depth_texture: texture::Depth,
    render_batches: Vec<RenderBatch>,
    ui_render_batches: Vec<UiRenderBatch>,
    text_writer: TextWriter,

    is_first_render: bool,
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub scale: cgmath::Matrix4<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw::new(self)
    }
}

// Hmm we might be able to batch together different meshes. group models based on shadows, lighting, transparency etc
// TODO Camera is also a bind group...
struct RenderBatch {
    instance_buffer: Buffer,
    model_id: String,
    instance_count: u32,
}

// maybe just like renderbatch, there should also be instances right? multiple swords in inventory lead to same draw?
struct UiRenderBatch {
    model_id: String,
    instance_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Renderer {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: InstanceFlags::empty(), // Disable Vulkan validation layers
            ..Default::default()
        });

        #[cfg(feature = "debug-logging")]
        log::debug!("Wgsl features: {:?}", instance.wgsl_language_features());

        let window_size = window.inner_size();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        #[cfg(feature = "debug-logging")]
        log::debug!("GPU adapter info: {:?}", adapter.get_info());
        #[cfg(feature = "debug-logging")]
        log::debug!("GPU Features: {:?}", adapter.features());
        #[cfg(feature = "debug-logging")]
        log::debug!("GPU limits: {:?}", adapter.limits());

        let desired_features = Self::create_gpu_compression_format_feature(&adapter);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_features: desired_features,
                required_limits: wgpu::Limits::defaults(), // Ideally only request what is needed: At some point make our own limits. 4k requires: `Surface` width and height must be within the maximum supported texture size. Requested was (3840, 2160), maximum extent for either dimension is 2048.
                memory_hints: MemoryHints::Performance,
                trace: Trace::default(),
            })
            .await
            .expect("Failed to create device. One must be available.");

        let capabilities = surface.get_capabilities(&adapter);
        #[cfg(feature = "debug-logging")]
        log::debug!("{capabilities:?}");
        let preferred_surface_texture_format = capabilities.formats[0]; // https://developer.chrome.com/blog/new-in-webgpu-127#dawn_updates "Instead, use wgpu::Surface::GetCapabilities() to get the list of supported formats, then use formats[0]"
        let mut view_formats = Vec::new();
        // "WebGPU doesn't support using sRGB texture formats as the output for a surface. " - https://sotrh.github.io/learn-wgpu/intermediate/tutorial13-hdr/#output-too-dark-on-webgpu.
        // Adding srgb view for webgpu to get right colours. When using config.format we need to add_srgb_suffix() as well
        if !preferred_surface_texture_format.is_srgb() {
            view_formats.push(preferred_surface_texture_format.add_srgb_suffix());
        }
        let present_mode = if capabilities.present_modes.contains(&Mailbox) {
            Mailbox
        } else {
            AutoVsync
        };
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: preferred_surface_texture_format,
            width: window_size.width.max(1),
            height: window_size.height.max(1),
            present_mode,
            alpha_mode: Auto,
            view_formats,
            desired_maximum_frame_latency: 1, // faster than default frame display. Guessing Chrome just always sets this to 2, because there's 1 frame extra delay according to performance tab
        };
        surface.configure(&device, &config);

        let camera_manager = CameraManager::new(&device);
        let model_manager = ModelManager::new().await;
        let primitive_vertices_manager = PrimitiveVerticesManager::new(&device);
        let color_manager = ColorManager::new(&device);
        let material_manager = TextureManager::new(&device, &queue);
        let render_context_manager = RenderContextManager::new(
            &device,
            &config,
            &color_manager,
            &camera_manager,
            &material_manager,
        );

        // Meh, configure + depth texture creation also happen in resize, which is called in web before rendering.
        let depth_texture = texture::Depth::create_depth_texture(&device, &config, "depth_texture");
        let text_writer = TextWriter::new(&device, &queue, &config).await;

        Self {
            surface,
            device,
            queue,
            config,
            model_manager,
            primitive_vertices_manager,
            color_manager,
            texture_manager: material_manager,
            camera_manager,
            render_context_manager,
            depth_texture,
            render_batches: Vec::new(),
            ui_render_batches: Vec::new(),
            text_writer,
            is_first_render: true,
        }
    }

    fn create_gpu_compression_format_feature(adapter: &Adapter) -> Features {
        let available_features = adapter.features();
        let mut desired_features = Features::empty();
        if available_features.contains(Features::TEXTURE_COMPRESSION_BC) {
            desired_features |= Features::TEXTURE_COMPRESSION_BC;
        } else {
            panic!("We expect BC compression to be supported right now. More support later")
        }
        desired_features
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Depth::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn render(&mut self, window: &Arc<Window>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Render view"),
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render_world(&view, &mut encoder);
        self.render_ui(&view, &mut encoder); // TODO maybe render UI first if we use stencil buffer

        self.queue.submit(iter::once(encoder.finish()));
        window.pre_present_notify();
        output.present();
        if self.is_first_render {
            window.set_visible(true);
            self.is_first_render = false;
        }

        Ok(())
    }

    fn render_world(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        for render_group in &self.render_batches {
            let model_definition = self.model_manager.get_model_3d(&render_group.model_id);
            let first_primitive = model_definition
                .primitives
                .first()
                .expect("Models have at least one primitive"); // We only render one primitive right now
            let primitive_vertices = self
                .primitive_vertices_manager
                .get_primitive_vertices(&first_primitive.vertices_id);
            let pipeline = &self
                .render_context_manager
                .render_contexts
                .get("3d")
                .expect("3d render pipeline exists");
            render_pass.set_pipeline(pipeline); // TODO there is only one pipeline atm... just set once?

            let color = self
                .color_manager
                .get_color_bind_group(&first_primitive.color_definition.id);
            let texture_bind_group = self
                .texture_manager
                .get_bind_group(first_primitive.texture_definition.as_ref());

            render_pass.set_bind_group(0, color, &[]);
            render_pass.set_bind_group(1, texture_bind_group, &[]);
            render_pass.set_bind_group(2, self.camera_manager.get_bind_group("camera_3d"), &[]);
            render_pass.set_vertex_buffer(0, primitive_vertices.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                primitive_vertices.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(
                0..primitive_vertices.num_indices,
                0,
                0..render_group.instance_count,
            );
        }
    }

    fn render_ui(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
        // TODO we could even fill the render pass earlier than starting of render... UI is not gonna be updated in between update and render i think...
        let mut render_pass_ui = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass UI"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let pipeline = &self
            .render_context_manager
            .render_contexts
            .get("ui")
            .expect("ui render pipeline exists");
        render_pass_ui.set_pipeline(pipeline);
        render_pass_ui.set_bind_group(2, self.camera_manager.get_bind_group("camera_2d"), &[]);

        self.draw_ui(&mut render_pass_ui);

        self.text_writer.write_text_buffer(&mut render_pass_ui);
    }

    fn create_instance_buffer(device: &Device, instance_group: &[Instance]) -> Buffer {
        let raw_instances = instance_group
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn convert_instance(
        position: &Point3<f32>,
        size: Option<&Scale>,
        rotation: Option<&crate::state::components::Rotation>,
    ) -> Instance {
        let scale = if let Some(size_unwrap) = size {
            cgmath::Vector4::new(size_unwrap.x, size_unwrap.y, size_unwrap.z, 1.0)
        } else {
            cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0)
        };
        Instance {
            position: Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            scale: cgmath::Matrix4::from_diagonal(scale),
            rotation: cgmath::Quaternion::from_axis_angle(
                Vector3::unit_y(),
                cgmath::Deg(rotation.map_or(0.0, |r| r.degrees_y)),
            ),
        }
    }

    pub fn create_ui_element_instance(window: &Arc<Window>, rect: &mut UIElement) -> Instance {
        rect.update(&window.inner_size());
        Instance {
            position: Vector3 {
                x: UIState::clip_space_element_position_x(rect, window),
                y: UIState::clip_space_element_position_y(rect, window),
                z: 0.0,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                UIState::convert_scale_x(rect.scaled_width, window),
                UIState::convert_scale_y(rect.scaled_height, window),
                1.0,
                1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0)),
        }
    }

    // TODO one of the most expensive methods. Maybe just check the diff of the game state and update the batches accordingly by removing/adding to batches
    fn create_render_batches(&mut self, game_state: &GameState) {
        let mut bind_group_entities: HashMap<String, Vec<String>> = HashMap::new();

        // TODO Group by identical bind groups instead of by model id
        // i think we have to iterate over each primitive in the model? though theoretically we can group primitives instead if they have different properties. shared textures for different models we probably would want to render same time in order not to change bind groups again
        game_state
            .entities
            .iter()
            .filter(|entity| game_state.get_position(entity).is_some())
            .filter(|entity| {
                game_state
                    .graphics_3d_components
                    .contains_key(entity.as_str())
            })
            .for_each(|entity| {
                let model_id = game_state
                    .get_graphics(entity)
                    .expect("Entity contains 3d component")
                    .model_id
                    .clone();
                bind_group_entities
                    .entry(model_id)
                    .or_default()
                    .push(entity.clone());
            });

        // TODO what is the difference again between groups and batch? naming?
        let mut render_batches: Vec<RenderBatch> = Vec::new();
        for (model_id, entity_group) in bind_group_entities.drain() {
            let instance_group: Vec<Instance> = entity_group
                .into_iter()
                .map(|entity| {
                    let size = game_state.get_size(&entity);
                    let rotation = game_state.get_rotation(&entity);
                    Self::convert_instance(
                        game_state.get_position(&entity).unwrap(),
                        size,
                        rotation,
                    )
                })
                .collect();
            let instance_buffer = Self::create_instance_buffer(&self.device, &instance_group);
            let render_batch = RenderBatch {
                instance_buffer,
                model_id,
                instance_count: instance_group.len() as u32,
            };
            render_batches.push(render_batch);
        }
        self.render_batches = render_batches;
    }

    fn update_camera_data_ui(&mut self, camera: &mut Camera, window: &Arc<Window>) {
        // TODO hmm maybe only needs called on resize?
        camera.update_view_projection_matrix(window); // TODO hmm i think camera matrix is updated in systems for 3d but for ui we do it here... one place for all.
        self.camera_manager
            .update_buffer("camera_2d", &self.queue, camera);
    }

    fn draw_ui(&mut self, render_pass: &mut RenderPass) {
        for render_batch in &self.ui_render_batches {
            let model = self.model_manager.get_model_2d(&render_batch.model_id);
            let primitive = model
                .primitives
                .first()
                .expect("Every model has one primitive right now");

            let color = self
                .color_manager
                .get_color_bind_group(&primitive.color_definition.id);

            let texture = self
                .texture_manager
                .get_bind_group(primitive.texture_definition.as_ref());

            render_pass.set_bind_group(0, color, &[]);
            render_pass.set_bind_group(1, texture, &[]);

            // let element_instance = Self::create_ui_element_instance(window, *ui_element);
            // let instance_buffer = Self::create_instance_buffer(&self.device, &[element_instance]); // TODO pretty expensive method call, can be done earlier. Don't generate buffers during rendering
            let instance_count = 1;

            let primitive_vertices = self
                .primitive_vertices_manager
                .get_primitive_vertices(&primitive.vertices_id);
            render_pass.set_vertex_buffer(0, primitive_vertices.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, render_batch.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                primitive_vertices.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..primitive_vertices.num_indices, 0, 0..instance_count);
        }
    }

    pub fn load_primitive_vertices_to_memory(
        &mut self,
        primitive_vertices: &Vec<PrimitiveVertices>,
    ) {
        for primitive_vertices in primitive_vertices {
            self.primitive_vertices_manager
                .load_primitive_vertices_to_memory(&self.device, primitive_vertices);
            self.model_manager.added_vertices(&primitive_vertices.name);
        }
    }

    pub fn load_color_to_memory(&mut self, color_definition: &ColorDefinition) {
        self.color_manager
            .load_color_to_memory(&self.device, color_definition);
        self.model_manager.added_color(&color_definition.id);
    }

    pub fn load_material_to_memory(&mut self, asset: &ImageAsset) {
        self.texture_manager
            .load_material_to_memory(&self.device, &self.queue, asset);
        self.model_manager.added_texture(&asset.name);
    }

    pub fn load_font_to_memory(&mut self, font: FontAsset) {
        self.text_writer.load_font_to_memory(font);
    }

    // TODO maybe also make sure render does not get called during this period
    pub fn updating(&mut self) {
        self.text_writer.reset_for_update();
    }

    pub fn updated(
        &mut self,
        window: &Arc<Window>,
        frame_state: &mut UpdateState,
        game_state: &mut GameState,
    ) {
        self.create_render_batches(game_state);

        let camera = game_state
            .camera_components
            .get_mut("camera_3d")
            .expect("Camera components should exist");
        self.camera_manager
            .update_buffer("camera_3d", &self.queue, camera);

        let camera = game_state
            .camera_components
            .get_mut("camera_ui")
            .expect("Camera components should exist");
        self.update_camera_data_ui(camera, window);

        frame_state
            .gui
            .render_commands
            .sort_by_key(|render_command| match render_command {
                RenderCommand::Model { layer, .. } | RenderCommand::Text { layer, .. } => *layer,
            });

        // TODO not true batches, all with 1 instance
        let mut ui_render_batches = Vec::new();
        for command in &mut frame_state.gui.render_commands {
            match command {
                RenderCommand::Model {
                    layer: _layer,
                    ui_element,
                    model_id,
                } => {
                    let element_instance = Self::create_ui_element_instance(window, ui_element);
                    let instance_buffer =
                        Self::create_instance_buffer(&self.device, &[element_instance]); // TODO pretty expensive method call, can be done earlier. Don't generate buffers during rendering
                    ui_render_batches.push(UiRenderBatch {
                        model_id: model_id.to_owned(),
                        instance_buffer,
                    })
                }
                RenderCommand::Text {
                    layer: _layer,
                    rect,
                    text,
                    color,
                } => {
                    // TODO expensive call, maybe only call upon changes
                    self.text_writer.add(window, rect, text, color);
                }
            }
        }
        self.ui_render_batches = ui_render_batches;
        self.text_writer.prepare(&self.device, &self.queue, window);
    }
}
