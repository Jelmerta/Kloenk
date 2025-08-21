use cgmath::{prelude::*, Point3, Vector3};
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use wgpu::{Buffer, CommandEncoder, Device, Features, InstanceFlags, MemoryHints, PresentMode, Queue, SurfaceConfiguration, TextureView, Trace};

use crate::application::ImageAsset;
use crate::render::camera::Camera;
use crate::render::camera_manager::CameraManager;
use crate::render::color_manager::ColorManager;
use crate::render::instance::InstanceRaw;
use crate::render::material_manager::TextureManager;
use crate::render::model::{ColorDefinition, Draw};
use crate::render::model_manager::ModelManager;
use crate::render::primitive_vertices_manager::{PrimitiveVertices, PrimitiveVerticesManager};
use crate::render::render_context_manager::RenderContextManager;
use crate::render::text_renderer::TextWriter;
use crate::render::texture;
use crate::state::components::Scale;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::{RenderCommand, UIElement, UIState};
use wgpu::util::DeviceExt;
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
    render_batches: Vec<RenderBatch>, // TODO Probably group by mesh otherwise we cannot batch? Also maybe this is a RenderBatch?
    text_writer: TextWriter,
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
struct RenderBatch {
    instance_buffer: Buffer,
    model_id: String,
    instance_count: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Renderer {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: InstanceFlags::empty(), // Remove Vulkan validation layer as this leads to tons of unhelpful logging (and VK_LAYER_KHRONOS_validation does not seem to exist? not debugging this)
            ..Default::default()
        });

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

        // Dev flag log?
        // let backend = adapter.get_info().backend;
        // match backend {
        //     Backend::Vulkan => log::info!("Using Vulkan backend"),
        //     Backend::Metal => log::info!("Using Metal backend"),
        //     Backend::Dx12 => log::info!("Using DirectX 12 backend"),
        //     Backend::Gl => log::info!("Using OpenGL backend (likely WebGL)"),
        //     Backend::BrowserWebGpu => log::info!("Using Browser's WebGPU backend"),
        //     Backend::Noop => log::info!("No graphics backend"),
        // }

        // Add gpu compression formats
        let available_features = adapter.features();
        let mut desired_features = Features::empty();
        if available_features.contains(Features::TEXTURE_COMPRESSION_BC) {
            desired_features |= Features::TEXTURE_COMPRESSION_BC;
        } else {
            panic!("We expect BC compression to be supported right now. More support later")
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: desired_features,
                required_limits: wgpu::Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: Trace::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width.max(1),
            height: window_size.height.max(1),
            // present_mode: surface_caps.present_modes[0],
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()], // Adding srgb view for webgpu. When using config.format we need to add_srgb_suffix() as well TODO also required on desktop? or only web?
            desired_maximum_frame_latency: 1, // faster than default frame display, probably falls back to 1. Guessing Chrome just always sets this to 2, because there's 1 frame extra delay according to performance tab
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
            text_writer,
        }
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

    pub fn render(
        &mut self,
        window: &Arc<Window>,
        game_state: &mut GameState,
        frame_state: &mut FrameState,
    ) -> Result<(), wgpu::SurfaceError> {
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

        self.render_world(game_state, &view, &mut encoder);
        self.render_ui(window, game_state, frame_state, &view, &mut encoder);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        self.text_writer.reset_for_frame();

        Ok(())
    }

    fn render_world(
        &mut self,
        game_state: &mut GameState,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.create_render_batches(game_state);

        let camera = game_state
            .camera_components
            .get_mut("camera")
            .expect("Camera components should exist");
        self.camera_manager
            .update_buffer("camera_3d", &self.queue, camera);

        {
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

            self.render_batches.drain(..).for_each(|render_group| {
                let model_definition = self.model_manager.get_model_3d(&render_group.model_id);
                let first_primitive = model_definition.primitives.first().expect("Models have at least one primitive"); // We only render one primitive right now
                let primitive_vertices = self
                    .primitive_vertices_manager
                    .get_primitive_vertices(&first_primitive.vertices_id);
                let pipeline = &self.render_context_manager.render_pipeline;
                render_pass.set_pipeline(pipeline);

                let color = self
                    .color_manager
                    .get_color_bind_group(&first_primitive.color_definition.id);
                let texture_bind_group = self
                    .texture_manager
                    .get_bind_group(first_primitive.texture_definition.as_ref());

                render_pass.set_bind_group(0, color, &[]);
                render_pass.set_bind_group(1, texture_bind_group, &[]);
                render_pass.set_bind_group(2, self.camera_manager.get_bind_group("camera_3d"), &[]);
                render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
                render_pass
                    .draw_primitive_instanced(primitive_vertices, 0..render_group.instance_count);
            });

            drop(render_pass);
        }
    }

    fn render_ui(
        &mut self,
        window: &Arc<Window>,
        game_state: &mut GameState,
        frame_state: &mut FrameState,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let camera = game_state
            .camera_components
            .get_mut("camera_ui")
            .expect("Camera components should exist");
        self.set_camera_data_ui(camera, window);

        frame_state
            .gui
            .render_commands
            .sort_by_key(|render_command| match render_command {
                RenderCommand::Texture { layer, .. } | RenderCommand::Text { layer, .. } => *layer,
            });

        for render_command in frame_state.gui.render_commands.drain(..) {
            match render_command {
                RenderCommand::Text {
                    layer: _layer,
                    rect,
                    text,
                    color,
                } => {
                    self.text_writer.add(window, &rect, &text, &color);
                }
                RenderCommand::Texture {
                    layer: _layer,
                    ui_element: rect,
                    model_id: texture_model_id, // TODO is this mesh or model?
                } => {
                    self.draw_ui(window, view, encoder, &texture_model_id, &rect);
                }
            } // TODO or maybe call it widget?
        }
        self.text_writer
            .write(&self.device, &self.queue, encoder, view, window);
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

    pub fn create_ui_element_instance(window: &Arc<Window>, rect: UIElement) -> Instance {
        Instance {
            position: Vector3 {
                x: UIState::clip_space_element_position_x(rect, window),
                y: UIState::convert_clip_space_y(rect.top()),
                z: 0.0,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                UIState::convert_scale_x(rect.width()),
                UIState::convert_scale_y(rect.height()),
                1.0,
                1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0)),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_render_batches(&mut self, game_state: &GameState) {
        // TODO what is the difference again? naming?
        let mut render_batches: Vec<RenderBatch> = Vec::new();
        let mut render_groups: HashMap<String, Vec<String>> = HashMap::new();

        game_state
            .entities
            .iter()
            .filter(|entity| game_state.get_position(entity.as_ref()).is_some())
            .filter(|entity| {
                game_state
                    .graphics_3d_components
                    .contains_key(entity.as_str())
            })
            .for_each(|entity| {
                let model_id = game_state.get_graphics(entity).expect("Entity contains 3d component").model_id.clone();
                render_groups.entry(model_id).or_default().push(entity.clone());
            });

        for (model_id, entity_group) in render_groups {
            // TODO? i think we have to iterate over each primitive in the model?
            let instance_group: Vec<Instance> = entity_group
                .into_iter()
                .map(|entity| {
                    let size = game_state.get_size(&entity);
                    let rotation = game_state.get_rotation(&entity);
                    Self::convert_instance(game_state.get_position(&entity).unwrap(), size, rotation)
                })
                .collect();
            let instance_buffer = Self::create_instance_buffer(&self.device, &instance_group);
            // let model = self.model_manager.get_model_3d(&model_id);
            // let primitive = model.primitives.first().expect("Models contain at least one primitive");
            let render_group = RenderBatch {
                instance_buffer,
                // Probably should be model when models have multiple primitives
                // TODO we should be able to consume rendergroups?
                model_id, // TODO i dont like cloning here... maybe pass an id to the primitive definition and then retrieve the whole definition from a map?
                // primitive: primitive.clone(), // TODO i dont like cloning here... maybe pass an id to the primitive definition and then retrieve the whole definition from a map?
                instance_count: instance_group.len() as u32,
            };
            render_batches.push(render_group);
        }
        self.render_batches = render_batches;
    }

    fn set_camera_data_ui(&mut self, camera: &mut Camera, window: &Arc<Window>) {
        camera.update_view_projection_matrix(window); // TODO hmm i think camera matrix is updated in systems for 3d but for ui we do it here... one place for all.
        self.camera_manager
            .update_buffer("camera_2d", &self.queue, camera);
    }

    fn draw_ui(
        &mut self,
        window: &Arc<Window>,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        model_id: &str,
        ui_element: &UIElement,
    ) {
        let model = self.model_manager.get_model_2d(model_id);
        let primitive = model.primitives.first().unwrap(); // todo multiple primitives

        let pipeline = &self.render_context_manager.render_pipeline;

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

        let color = self
            .color_manager
            .get_color_bind_group(&primitive.color_definition.id);

        let texture = self
            .texture_manager
            .get_bind_group(primitive.texture_definition.as_ref()); // TODO consume instead of clone? just pass id?

        render_pass_ui.set_pipeline(pipeline);
        render_pass_ui.set_bind_group(0, color, &[]);
        render_pass_ui.set_bind_group(1, texture, &[]);
        render_pass_ui.set_bind_group(2, self.camera_manager.get_bind_group("camera_2d"), &[]);

        let element_instance = Self::create_ui_element_instance(window, *ui_element);
        let instance_buffer = Self::create_instance_buffer(&self.device, &[element_instance]);
        let instance_count = 1;

        render_pass_ui.set_vertex_buffer(1, instance_buffer.slice(..));
        let primitive_vertices = self
            .primitive_vertices_manager
            .get_primitive_vertices(&primitive.vertices_id);
        render_pass_ui.draw_primitive_instanced(primitive_vertices, 0..instance_count);
        drop(render_pass_ui);
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
}
