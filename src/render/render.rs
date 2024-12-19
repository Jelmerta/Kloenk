use cgmath::{prelude::*, Point3, Vector3};
use itertools::Itertools;
use std::iter;
use std::sync::Arc;
use wgpu::{
    Buffer, CommandEncoder, Device, InstanceFlags, MemoryHints, Queue, SurfaceConfiguration,
    TextureView,
};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::camera::Camera;
use crate::render::camera_manager::CameraManager;
use crate::render::instance::InstanceRaw;
use crate::render::material_manager::MaterialManager;
use crate::render::model::VertexType::Color;
use crate::render::model::{Draw, VertexType};
use crate::render::model_manager::ModelManager;
use crate::render::render_context_manager::RenderContextManager;
use crate::render::text_renderer::TextWriter;
use crate::render::texture;
use crate::state::components::{Entity, Size};
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::{RenderCommand, UIElement, UIState};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    model_manager: ModelManager,
    camera_manager: CameraManager,
    material_manager: MaterialManager,
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
    mesh_id: String,
    instance_count: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Renderer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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

        let backend = adapter.get_info().backend;
        match backend {
            wgpu::Backend::Empty => log::info!("No graphics backend"),
            wgpu::Backend::Vulkan => log::info!("Using Vulkan backend"),
            wgpu::Backend::Metal => log::info!("Using Metal backend"),
            wgpu::Backend::Dx12 => log::info!("Using DirectX 12 backend"),
            wgpu::Backend::Gl => log::info!("Using OpenGL backend (likely WebGL)"),
            wgpu::Backend::BrowserWebGpu => log::info!("Using Browser's WebGPU backend"),
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: MemoryHints::default(),
                },
                None,
            )
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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()], // Adding srgb view for webgpu. When using config.format we need to add_srgb_suffix() as well
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let model_manager = ModelManager::new(&device).await;
        let camera_manager = CameraManager::new(&device);
        let material_manager = MaterialManager::new(&device, &queue).await;
        let render_context_manager =
            RenderContextManager::new(&device, &config, &camera_manager, &material_manager);

        // Meh, configure + depth texture creation also happen in resize, which is called in web before rendering.
        let depth_texture = texture::Depth::create_depth_texture(&device, &config, "depth_texture");
        let text_writer = TextWriter::new(&device, &queue, &config).await;

        Self {
            surface,
            device,
            queue,
            config,
            model_manager,
            camera_manager,
            material_manager,
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
        frame_state: &FrameState,
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

        let camera = game_state.camera_components.get_mut("camera").unwrap();
        self.camera_manager
            .update_buffer("camera_3d".to_string(), &self.queue, camera);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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

            self.render_batches.iter().for_each(|render_group| {
                let mesh = self
                    .model_manager
                    .get_mesh(render_group.mesh_id.to_string());
                match &mesh.vertex_type {
                    Color { color: _color } => {
                        let render_context_colored = self
                            .render_context_manager
                            .render_contexts
                            .get_mut("colored")
                            .unwrap();
                        render_pass.set_pipeline(&render_context_colored.render_pipeline);
                        render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
                        render_pass.set_bind_group(
                            0,
                            self.camera_manager.get_bind_group("camera_3d"),
                            &[],
                        );
                    }
                    VertexType::Material { material_id } => {
                        let render_context_textured = self
                            .render_context_manager
                            .render_contexts
                            .get_mut("textured")
                            .unwrap();
                        render_pass.set_pipeline(&render_context_textured.render_pipeline);

                        let texture_bind_group = self.material_manager.get_bind_group(material_id);
                        render_pass.set_bind_group(0, texture_bind_group, &[]);
                        render_pass.set_bind_group(
                            1,
                            self.camera_manager.get_bind_group("camera_3d"),
                            &[],
                        );
                        render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
                    }
                }
                render_pass.draw_mesh_instanced(mesh, 0..render_group.instance_count);
            });

            drop(render_pass);
        }
    }

    fn render_ui(
        &mut self,
        window: &Arc<Window>,
        game_state: &mut GameState,
        frame_state: &FrameState,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let camera = game_state.camera_components.get_mut("camera_ui").unwrap();
        self.set_camera_data_ui(camera, &window);

        frame_state
            .gui
            .render_commands
            .iter()
            .sorted_by_key(|render_command| match render_command {
                RenderCommand::Mesh {
                    layer,
                    ui_element: _rect,
                    mesh_id: _image_name,
                } => layer,
                RenderCommand::Text {
                    layer,
                    rect: _rect,
                    text: _text,
                    color: _color,
                } => layer,
            })
            .for_each(|render_command| {
                match render_command {
                    RenderCommand::Text {
                        layer: _layer,
                        rect,
                        text,
                        color,
                    } => {
                        self.text_writer.add(&window, rect, text, color);
                    }
                    RenderCommand::Mesh {
                        layer: _layer,
                        ui_element: rect,
                        mesh_id,
                    } => {
                        self.draw(window, view, encoder, mesh_id.to_string(), rect);
                    }
                } // TODO or maybe call it widget?
            });
        self.text_writer
            .write(&self.device, &self.queue, encoder, view, window)
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
        size: Option<&Size>,
        rotation: Option<&crate::state::components::Rotation>,
    ) -> Instance {
        let scale = if let Some(size_unwrap) = size {
            cgmath::Vector4::new(
                size_unwrap.scale_x,
                size_unwrap.scale_y,
                size_unwrap.scale_z,
                1.0,
            )
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
                cgmath::Deg(rotation.map(|r| r.degrees_y).unwrap_or(0.0)),
            ),
        }
    }

    pub fn create_ui_element_instance(window: &Arc<Window>, rect: UIElement) -> Instance {
        Instance {
            position: Vector3 {
                x: UIState::clip_space_left(rect, window),
                // y: UIState::convert_clip_space_y(rect.top_left().y), // TODO hm middle adjusted or top left fine right?
                // y: UIState::convert_clip_space_y(UIState::), // TODO hm middle adjusted or top left fine right?
                y: rect.clip_top(),
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
        let mut render_batches: Vec<RenderBatch> = Vec::new();
        game_state
            .entities
            .iter()
            .filter(|entity| game_state.get_position(&(*entity).to_string()).is_some())
            .filter(|entity| {
                game_state
                    .graphics_3d_components
                    .contains_key(entity.as_str())
            })
            .chunk_by(|entity| {
                game_state
                    .get_graphics(&(*entity).to_string())
                    .unwrap()
                    .mesh_id
                    .clone()
            })
            .into_iter()
            .for_each(|(mesh_id, group)| {
                let entity_group: Vec<&Entity> = group.collect();
                let instance_group: Vec<Instance> = entity_group
                    .into_iter()
                    .map(|entity| {
                        let size = game_state.get_size(entity);
                        let rotation = game_state.get_rotation(entity);
                        Self::convert_instance(
                            game_state.get_position(&entity.to_string()).unwrap(),
                            size,
                            rotation,
                        )
                    })
                    .collect();
                let instance_buffer = Self::create_instance_buffer(&self.device, &instance_group);
                let render_group = RenderBatch {
                    instance_buffer,
                    mesh_id,
                    instance_count: instance_group.len() as u32,
                };
                render_batches.push(render_group);
            });
        self.render_batches = render_batches;
    }

    fn set_camera_data_ui(&mut self, camera: &mut Camera, window: &Arc<Window>) {
        camera.update_view_projection_matrix(window); // TODO hmm i think camera matrix is updated in systems for 3d but for ui we do it here... one place for all.
        self.camera_manager
            .update_buffer("camera_2d".to_string(), &self.queue, camera);
    }

    fn draw(
        &mut self,
        window: &Arc<Window>,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        mesh_id: String,
        ui_element: &UIElement,
    ) {
        let mesh = self.model_manager.get_mesh(mesh_id.to_string());

        match &mesh.vertex_type {
            Color { color: _color } => {
                let render_context_textured = self
                    .render_context_manager
                    .render_contexts
                    .get_mut("colored")
                    .unwrap();

                let mut render_pass_ui = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass UI"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    // TODO probably doesnt work on multiple render passes... Might need to rethink depth buffer on multi-renderpass
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

                render_pass_ui.set_pipeline(&render_context_textured.render_pipeline);
                render_pass_ui.set_bind_group(
                    0,
                    self.camera_manager.get_bind_group("camera_2d"),
                    &[],
                );

                let element_instance = Self::create_ui_element_instance(window, *ui_element);
                let instance_buffer =
                    Self::create_instance_buffer(&self.device, &[element_instance]);
                let instance_count = 1;

                render_pass_ui.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass_ui.draw_mesh_instanced(mesh, 0..instance_count);
                drop(render_pass_ui);
            }
            VertexType::Material { material_id } => {
                let render_context_textured = self
                    .render_context_manager
                    .render_contexts
                    .get_mut("textured")
                    .unwrap();

                let mut render_pass_ui = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass UI"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
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

                render_pass_ui.set_pipeline(&render_context_textured.render_pipeline);
                render_pass_ui.set_bind_group(
                    1,
                    self.camera_manager.get_bind_group("camera_2d"),
                    &[],
                );

                let element_instance = Self::create_ui_element_instance(window, *ui_element);
                let instance_buffer =
                    Self::create_instance_buffer(&self.device, &[element_instance]);
                let instance_count = 1;

                let material = &self
                    .material_manager
                    .get_material(material_id)
                    .texture_bind_group;
                render_pass_ui.set_bind_group(0, material, &[]);
                render_pass_ui.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass_ui.draw_mesh_instanced(mesh, 0..instance_count);
                drop(render_pass_ui);
            }
        }
    }
}
