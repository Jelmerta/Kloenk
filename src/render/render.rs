// use anyhow::*;
use cgmath::{prelude::*, Point2, Point3, Vector3};
use itertools::Itertools;
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, InstanceFlags, MemoryHints,
    PipelineCompilationOptions, Queue, RenderPass, RenderPipeline, ShaderModule,
    SurfaceConfiguration, TextureView,
};
// use gltf::iter::Meshes;
// use gltf::mesh::util::indices;
// use gltf::texture as gltf_texture;
// use gltf::Gltf;
//

// #[cfg(target_arch = "wasm32")]
// #[allow(unused_imports)]
// use wasm_bindgen::prelude::*;

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::components::{Entity, Size};
use crate::render::camera::Camera;
use crate::render::model::VertexType::Color;
use crate::render::model::{Draw, Material, Mesh, Vertex, VertexType};
use crate::render::text_renderer::TextWriter;
use crate::render::{model, texture};
use crate::resources;
use crate::resources::load_texture;
use crate::state::frame_state::FrameState;
use crate::state::game_state::GameState;
use crate::state::ui_state::{Rect, RenderCommand, UIState, WindowSize};
// #[wasm_bindgen(start)]
// pub fn run() -> Result<(), JsValue> {
//     let gltf_data = include_bytes!("../models/garfield/scene.gltf");
//     let gltf = Gltf::from_slice(gltf_data).expect("Failed to load Garfield");
//     log::debug!("{:?}", gltf);
//     Ok(())
// }

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_projection(&mut self, camera: &mut Camera) {
        self.view_projection = camera.view_projection_matrix.into();
    }
}

pub struct RenderContext {
    pub render_pipeline: RenderPipeline,
    camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    render_contexts: HashMap<String, RenderContext>,

    // models: Vec<model::Model>,
    //obj_model: model::Model,
    mesh_map: HashMap<String, Mesh>,
    material_map: HashMap<String, Material>,
    depth_texture: texture::Depth,
    render_batches: Vec<RenderBatch>, // TODO Probably group by mesh otherwise we cannot batch? Also maybe this is a RenderBatch?
    text_writer: TextWriter,
}

struct Instance {
    position: Vector3<f32>,
    scale: cgmath::Matrix4<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * self.scale
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
            ],
        }
    }
}

struct RenderBatch {
    instance_buffer: Buffer,
    mesh_id: String,
    instance_count: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, window_width: u32, window_height: u32) -> Renderer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: InstanceFlags::empty(), // Remove Vulkan validation layer as this leads to tons of unhelpful logging (and VK_LAYER_KHRONOS_validation does not seem to exist? not debugging this)
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

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
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_width,
            height: window_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout = Self::setup_texture_layout(&device);

        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../texture_shader.wgsl").into()),
        });

        let mut render_contexts = HashMap::new();
        render_contexts.insert(
            "render_context_3d_textured".to_string(),
            Self::setup_textured_3d_context(
                &device,
                &config,
                &texture_bind_group_layout,
                &texture_shader,
            ),
        );

        render_contexts.insert(
            "render_context_ui_textured".to_string(),
            Self::setup_textured_ui_context(
                &device,
                &config,
                &texture_bind_group_layout,
                &texture_shader,
            ),
        );

        // let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Color Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_str!("../color_shader.wgsl").into()),
        // });

        // let vertex_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Vertex Buffer"),
        //         contents: bytemuck::cast_slice(TRIANGLE),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     }
        // );
        // let num_vertices = TRIANGLE.len() as u32;
        // let (document, buffers, images) = gltf::import("examples/Box.gltf")?;
        // let gltf_data = include_bytes!("../models/garfield/scene.gltf");
        // let gltf = Gltf::from_slice(gltf_data).expect("Failed to load Garfield");
        // log::warn!("{:?}", gltf.scenes());
        //     log::warn!("Hi?");
        //     // let gltf = Gltf::open("models/garfield/scene.gltf").expect("Failed to load garfield kartfield");
        //     // log::debug!("{:?}", gltf);
        //     // for scene in gltf.scenes() {
        //     //     for node in scene.nodes() {
        //     //       println!(
        //     //       "Node #{} has {} children",
        //     //       node.index(),
        //     //          node.children().count(),
        //     //       );
        //     //     }
        //     // }
        //     //

        // let mut vertices: Vec<TexVertex> = Vec::new();
        //     let mut buffer_data = Vec::new();
        //     // buffer_data.push("2312".as_bytes());
        //
        //

        // Barely know what the buffers do yet...
        // for buffer in gltf.buffers() {
        //     match buffer.source() {
        //         gltf::buffer::Source::Bin => {
        //             // if let Some(blob) = gltf.blob.as_deref() {
        //             //     buffer_data.push(blob.into());
        //             //     println!("Found a bin, saving");
        //         // };
        //         }
        //         gltf::buffer::Source::Uri(uri) => {
        //             let bin = load_binary(uri).await; // TODO Tutorial does "await?" instead...
        //             // What am i missing
        //             buffer_data.push(bin);
        //         }
        //     }
        // }

        // let mut buffer_data = Vec::new();
        //     for buffer in gltf.buffers() {
        //         match buffer.source() {
        //             gltf::buffer::Source::Uri(uri) => {
        //                 // let uri = percent_encoding::percent_decode_str(uri)
        //                     // .decode_utf8()
        //                     // .unwrap();
        //                 // let uri = uri.as_ref();
        //                 // let buffer_bytes = match DataUri::parse(uri) {
        //                 //     Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => {
        //                         // data_uri.decode()?
        //                     // }
        //                     // Ok(_) => return Err(GltfError::BufferFormatUnsupported),
        //                     // Err(()) => {
        //                         // TODO: Remove this and add dep
        //                         // let buffer_path = load_context.path().parent().unwrap().join(uri);
        //                         // load_context.read_asset_bytes(buffer_path).await?
        //                     // }
        //                 // };
        //                 // buffer_data.push();
        //             }
        //             gltf::buffer::Source::Bin => {
        //                 if let Some(blob) = gltf.blob.as_deref() {
        //                     buffer_data.push(blob.into());
        //                 } else {
        //                     panic!(":)");
        //                 }
        //             }
        //         }
        //     }

        // let mut meshes = Vec::new();
        // for mesh in gltf.meshes() {
        //             // log::warn!("Mesh: {}", mesh.name().unwrap_or("Unnamed").into());
        //     // for primitive in mesh.primitives() {
        //         // let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
        //         // let positions: Vec<[f32; 3]> = if let Some(positions_accessor) = primitive.get(&gltf::Semantic::Positions) { // Hard to read imo
        //             // let reader = positions_accessor.reader();
        // //             reader.into_f32().map(|p| [p[0], p[1], p[2]]).collect()
        // //         } else {
        // //             vec![]
        // //         };
        // //     }
        // // }
        //     // }
        //     //
        //
        //     mesh.primitives().for_each(|primitive| {
        //         let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
        //     // let reader = primitive.reader(|buffer| Some(buffer_data[buffer.index()].as_slice()));

        // let mut vertices = Vec::new();
        // if let Some(vertex_attibute) = reader.read_positions() {
        //     vertex_attibute.for_each(|vertex| {
        //         vertices.push(TexVertex {
        //             position: vertex,
        //             tex_coords: Default::default(),
        //         })
        //     });
        // }

        // if let Some(normal_attribute) = reader.read_normals()

        // if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|tex_coord_index| tex_coord_index.into_f32()) { // We map so that
        //     let mut tex_coord_index = 0;
        //     tex_coord_attribute.for_each(|tex_coord| {
        //         vertices[tex_coord_index].tex_coords = tex_coord;

        //         tex_coord_index += 1; // does ++ not work?
        //     });
        // // we can increase the index of tex coords accordingly
        // }

        // let mut indices = Vec::new();
        // if let Some(indices_raw) = reader.read_indices() {
        //     indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
        // }

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex buffer"),
        //     contents: bytemuck::cast_slice(&vertices),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        //         meshes.push(model::Mesh {
        //             name: "Garfield".to_string(),
        //             vertex_buffer,
        //             index_buffer,
        //             num_elements: indices.len() as u32,
        //             material: 0,
        //         })
        //     });
        // }
        // //
        // // gltf.materials()
        // // //
        // // // let mut materials = Vec::new();
        // // for obj_material in object_materials? {
        //     // gltf.1
        //
        //     // let diffuse_texture = load_tex;
        //     // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     //     layout,
        //     //     entries: &[
        //     //         wgpu::BindGroupEntry {
        //     //             binding: 0,
        //     //             resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
        //     //         },
        //     //         wgpu::BindGroupEntry {
        //     //             binding: 1,
        //     //             resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
        //     //         },
        //     //     ],
        //     //     label: None,
        //     // });

        //     materials.push((model::Material {
        //         name: ,
        //         diffuse_texture,
        //         bind_group,
        //     });
        // }

        // let garfield = model::Model {
        //     meshes: meshes,
        //     materials: materials,
        // };
        // // models.push(garfield);

        let mesh_map = Self::load_models(&device).await;
        let material_map = Self::load_materials(&device, &queue, &texture_bind_group_layout).await;

        let depth_texture = texture::Depth::create_depth_texture(&device, &config, "depth_texture");
        let text_writer = TextWriter::new(
            &device,
            &queue,
            &surface,
            &adapter,
            window_width as f32,
            window_height as f32,
        )
        .await;
        Self {
            surface,
            device,
            queue,
            config,
            size: PhysicalSize::new(window_width, window_height),
            render_contexts,
            mesh_map,
            material_map,
            //obj_model: garfield,
            depth_texture,
            render_batches: Vec::new(),
            text_writer,
        }
    }

    fn setup_texture_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    fn setup_textured_3d_context(
        device: &Device,
        config: &SurfaceConfiguration,
        texture_bind_group_layout: &BindGroupLayout,
        shader: &ShaderModule,
    ) -> RenderContext {
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[model::TexVertex::desc(), InstanceRaw::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Depth::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        RenderContext {
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
        }
    }

    fn setup_textured_ui_context(
        device: &Device,
        config: &SurfaceConfiguration,
        texture_bind_group_layout: &BindGroupLayout,
        texture_shader: &ShaderModule,
    ) -> RenderContext {
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: texture_shader,
                entry_point: Some("vs_main"),
                buffers: &[model::TexVertex::desc(), InstanceRaw::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: texture_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        RenderContext {
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
        }
    }

    async fn load_models(device: &Device) -> HashMap<String, Mesh> {
        let mut mesh_map: HashMap<String, Mesh> = HashMap::new();
        let shield = resources::load_model(device, "CUBE", "shield")
            .await
            .unwrap();
        mesh_map.insert(
            "shield".to_string(),
            shield.meshes.into_iter().next().unwrap(),
        );

        let shield_inventory = resources::load_model(device, "SQUARE", "shield")
            .await
            .unwrap();
        mesh_map.insert(
            "shield_inventory".to_string(),
            shield_inventory.meshes.into_iter().next().unwrap(),
        );

        let character = resources::load_model(device, "CUBE", "character")
            .await
            .unwrap();
        mesh_map.insert(
            "character".to_string(),
            character.meshes.into_iter().next().unwrap(),
        );

        let sword = resources::load_model(device, "CUBE", "sword")
            .await
            .unwrap();
        mesh_map.insert(
            "sword".to_string(),
            sword.meshes.into_iter().next().unwrap(),
        );

        let sword_inventory = resources::load_model(device, "SQUARE", "sword")
            .await
            .unwrap();
        mesh_map.insert(
            "sword_inventory".to_string(),
            sword_inventory.meshes.into_iter().next().unwrap(),
        );

        let grass = resources::load_model(device, "CUBE", "grass")
            .await
            .unwrap();
        mesh_map.insert(
            "grass".to_string(),
            grass.meshes.into_iter().next().unwrap(),
        );

        let tree = resources::load_model(device, "CUBE", "tree").await.unwrap();
        mesh_map.insert("tree".to_string(), tree.meshes.into_iter().next().unwrap());
        mesh_map
    }

    async fn load_materials(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
    ) -> HashMap<String, Material> {
        let mut materials = HashMap::new();
        materials.insert(
            "sword".to_string(),
            Self::load_material(device, queue, layout, "sword.jpg")
                .await
                .unwrap(),
        );
        materials.insert(
            "shield".to_string(),
            Self::load_material(device, queue, layout, "shield.jpg")
                .await
                .unwrap(),
        );
        materials.insert(
            "character".to_string(),
            Self::load_material(device, queue, layout, "character.jpg")
                .await
                .unwrap(),
        );
        materials.insert(
            "grass".to_string(),
            Self::load_material(device, queue, layout, "grass.jpg")
                .await
                .unwrap(),
        );
        materials.insert(
            "tree".to_string(),
            Self::load_material(device, queue, layout, "tree.png")
                .await
                .unwrap(),
        );
        materials
    }

    async fn load_material(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        file_name: &str,
    ) -> anyhow::Result<Material> {
        let diffuse_texture = load_texture(file_name, device, queue).await?;
        let bind_group = Self::build_bind_group(device, layout, &diffuse_texture);
        Ok(Material {
            texture_bind_group: bind_group,
        })
    }

    fn build_bind_group(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        diffuse_texture: &texture::Texture,
    ) -> BindGroup {
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        diffuse_bind_group
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Depth::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn render(
        &mut self,
        game_state: &mut GameState,
        ui_state: &UIState,
        frame_state: &FrameState,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render_world(game_state, &view, &mut encoder);

        self.render_ui(game_state, ui_state, frame_state, &view, &mut encoder);

        //use model::DrawModel;
        // let garfield = self.models.pop().unwrap();
        // let mesh = &garfield.meshes[0];
        // render_pass.draw_mesh_instanced(&garfield.meshes[0].clone(), 0..instances.len() as u32);
        //render_pass.draw_model_instanced(&self.obj_model, 0..instances.len() as u32);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_world(
        &mut self,
        game_state: &mut GameState,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.create_render_groups(game_state);

        let render_context_3d_textured = self
            .render_contexts
            .get_mut("render_context_3d_textured")
            .unwrap();

        let camera = game_state.camera_components.get_mut("camera").unwrap();
        render_context_3d_textured
            .camera_uniform
            .update_view_projection(camera);
        self.queue.write_buffer(
            &render_context_3d_textured.camera_buffer,
            0,
            bytemuck::cast_slice(&[render_context_3d_textured.camera_uniform]),
        );

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

            render_pass.set_pipeline(&render_context_3d_textured.render_pipeline);
            render_pass.set_bind_group(1, &render_context_3d_textured.camera_bind_group, &[]);

            self.render_batches.iter().for_each(|render_group| {
                let mesh = &self.mesh_map.get(&render_group.mesh_id).unwrap();
                match &mesh.vertex_type {
                    Color { color: _color } => (),
                    VertexType::Texture { material_id } => {
                        let material = self.material_map.get(material_id).unwrap();
                        render_pass.set_bind_group(0, &material.texture_bind_group, &[]);
                        render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
                        render_pass.draw_mesh_instanced(mesh, 0..render_group.instance_count);
                    }
                }
            });

            drop(render_pass);
        }
    }

    fn render_ui(
        &mut self,
        game_state: &mut GameState,
        ui_state: &UIState,
        frame_state: &FrameState,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let camera = game_state.camera_components.get_mut("camera_ui").unwrap();
        self.set_camera_data_ui(camera, &ui_state.window_size);

        frame_state
            .gui
            .render_commands
            .iter()
            .sorted_by_key(|render_command| match render_command {
                RenderCommand::Image {
                    layer,
                    rect: _rect,
                    mesh_id: _image_name,
                } => layer,
                RenderCommand::Text {
                    layer,
                    rect: _rect,
                    text: _text,
                } => layer,
            })
            .for_each(|render_command| {
                match render_command {
                    RenderCommand::Text {
                        layer: _layer,
                        rect,
                        text,
                    } => {
                        self.text_writer.prepare(
                            &self.device,
                            &self.queue,
                            ui_state.window_size.width,
                            ui_state.window_size.height,
                            rect.scale(
                                ui_state.window_size.width as f32,
                                ui_state.window_size.height as f32,
                            ),
                        );

                        self.text_writer.write_text_buffer(encoder, view, text);
                    }
                    RenderCommand::Image {
                        layer: _layer,
                        rect,
                        mesh_id,
                    } => {
                        let render_context_ui_textured = self
                            .render_contexts
                            .get_mut("render_context_ui_textured")
                            .unwrap();

                        let mut render_pass_ui =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass UI"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view,
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

                        render_pass_ui.set_pipeline(&render_context_ui_textured.render_pipeline);
                        render_pass_ui.set_bind_group(
                            1,
                            &render_context_ui_textured.camera_bind_group,
                            &[],
                        );

                        let element_instance = Self::create_ui_element_instance(
                            Point2::new(
                                ui_state.window_size.width as f32,
                                ui_state.window_size.height as f32,
                            ),
                            *rect,
                        );
                        let element_render_group = RenderBatch {
                            instance_buffer: Self::create_instance_buffer(
                                &self.device,
                                &[element_instance],
                            ),
                            mesh_id: mesh_id.to_string(),
                            instance_count: 1,
                        };
                        self.draw_ui(&mut render_pass_ui, &element_render_group);

                        drop(render_pass_ui);
                    }
                } // TODO or maybe call it widget?
            });
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

    fn convert_instance(position: &Point3<f32>, size: Option<&Size>) -> Instance {
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
            rotation: cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0)),
        }
    }

    fn create_ui_element_instance(window_dimensions: Point2<f32>, rect: Rect) -> Instance {
        Instance {
            position: Vector3 {
                x: UIState::convert_clip_space_x(
                    rect.top_left.x,
                    window_dimensions.x,
                    window_dimensions.y,
                ),
                y: UIState::convert_clip_space_y(rect.top_left.y),
                z: 0.0,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                UIState::convert_scale_x(rect.width(), window_dimensions.x, window_dimensions.y),
                UIState::convert_scale_y(rect.height()),
                1.0,
                1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(Vector3::unit_z(), cgmath::Deg(0.0)),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_render_groups(&mut self, game_state: &GameState) {
        let mut render_groups: Vec<RenderBatch> = Vec::new();
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
                // "group_by"
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
                        Self::convert_instance(
                            game_state.get_position(&entity.to_string()).unwrap(),
                            size,
                        )
                    })
                    .collect();
                let instance_buffer = Self::create_instance_buffer(&self.device, &instance_group);
                let render_group = RenderBatch {
                    instance_buffer,
                    mesh_id,
                    instance_count: instance_group.len() as u32,
                };
                render_groups.push(render_group);
            });
        self.render_batches = render_groups;
    }

    fn set_camera_data_ui(&mut self, camera: &mut Camera, window_size: &WindowSize) {
        let render_context_ui_textured = self
            .render_contexts
            .get_mut("render_context_ui_textured")
            .unwrap();

        camera.update_view_projection_matrix(window_size.width, window_size.height);
        render_context_ui_textured
            .camera_uniform
            .update_view_projection(camera);
        self.queue.write_buffer(
            &render_context_ui_textured.camera_buffer,
            0,
            bytemuck::cast_slice(&[render_context_ui_textured.camera_uniform]),
        );
    }

    fn draw_ui<'a>(&'a self, render_pass: &mut RenderPass<'a>, render_group: &RenderBatch) {
        let mesh = &self.mesh_map.get(&render_group.mesh_id).unwrap();
        match &mesh.vertex_type {
            Color { color: _ } => {}
            VertexType::Texture { material_id } => {
                let material = self.material_map.get(material_id).unwrap();
                render_pass.set_bind_group(0, &material.texture_bind_group, &[]);
                render_pass.set_vertex_buffer(1, render_group.instance_buffer.slice(..));
                render_pass.draw_mesh_instanced(mesh, 0..render_group.instance_count);
            }
        }
    }
}
