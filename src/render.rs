use itertools::Itertools;
use wgpu::RenderPass;
use std::collections::HashMap;
use std::{iter, mem};
// use anyhow::*;
use cgmath::{prelude::*, Point3};
// use gltf::iter::Meshes;
// use gltf::mesh::util::indices;
// use gltf::texture as gltf_texture;
// use gltf::Gltf;
//

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::camera::Camera;
use crate::components::{Entity, InStorage, Position};
use crate::game_state::GameState;
use crate::gui::UIState;
use crate::model::Vertex;
use crate::model::{self};
use crate::{resources, texture};
use model::DrawModel;
// use crate::resources::load_binary;

// #[wasm_bindgen(start)]
// pub fn run() -> Result<(), JsValue> {
//     let gltf_data = include_bytes!("../models/garfield/scene.gltf");
//     let gltf = Gltf::from_slice(gltf_data).expect("Failed to load Garfield");
//     log::debug!("{:?}", gltf);
//     Ok(())
// }

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
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

    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_ui: wgpu::RenderPipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_ui: Camera,
    camera_uniform_ui: CameraUniform,
    camera_buffer_ui: wgpu::Buffer,
    camera_bind_group_ui: wgpu::BindGroup,
    // models: Vec<model::Model>,
    //obj_model: model::Model,
    model_map: HashMap<String, model::Model>,
    depth_texture: texture::DepthTexture,
    render_groups: Vec<RenderGroup>,
}

struct Instance {
    position: cgmath::Vector3<f32>,
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
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
            ],
        }
    }
}

struct RenderGroup {
    buffer: wgpu::Buffer,
    model_id: String,
    instance_count: u32,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // The instance is a handle to our GPU
            backends: wgpu::Backends::all(),
            //=> Vulkan + Metal + DX12 + Browser WebGPU
            // #[cfg(not(target_arch = "wasm32"))]
            // backends: wgpu::Backends::PRIMARY,
            // #[cfg(target_arch = "wasm32")]
            // backends: wgpu::Backends::GL,
            // backends: wgpu::Backends::SECONDARY,
            // backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let limits = adapter.limits();
        log::warn!("{:?}", limits);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // required_limits: wgpu::Limits::downlevel_defaults(),
                    // required_limits: if cfg!(target_arch = "wasm32") {
                    // required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    // } else {
                    required_limits: wgpu::Limits::default(),
                    // },
                    memory_hints: Default::default(),
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
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // ----- texture stuff
        let texture_bind_group_layout =
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
            });

        // ----- end texture stuff

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera = Camera::new();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, // what is COPY_DST?
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        // Binding type, horrible naming
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
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                // buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                buffers: &[model::TexVertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
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
            // depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DepthTexture::DEPTH_FORMAT,
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
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let camera_ui = Camera::new();

        let mut camera_uniform_ui = CameraUniform::new();
        camera_uniform_ui.update_view_projection(&camera_ui);

        let camera_buffer_ui = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer UI"),
            contents: bytemuck::cast_slice(&[camera_uniform_ui]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, // what is COPY_DST?
        });

        let camera_bind_group_layout_ui =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout UI"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        // Binding type, horrible naming
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group_ui = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group UI"),
            layout: &camera_bind_group_layout_ui,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer_ui.as_entire_binding(),
            }],
        });

        let render_pipeline_layout_ui =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout UI"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout_ui],
                push_constant_ranges: &[],
            });

        let render_pipeline_ui = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline UI"),
            layout: Some(&render_pipeline_layout_ui),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                // buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                buffers: &[model::TexVertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
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
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

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

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(CUBE),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        let mut model_map: HashMap<String, model::Model> = HashMap::new();

        let shield = resources::load_model(
            "resources/shield.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "CUBE",
        )
        .await
        .unwrap();
        model_map.insert("shield".to_string(), shield);

        let shield_inventory = resources::load_model(
            "resources/shield.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "SQUARE",
        )
        .await
        .unwrap();
        model_map.insert("shield_inventory".to_string(), shield_inventory);

        let character = resources::load_model(
            "resources/character.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "CUBE",
        )
        .await
        .unwrap();
        model_map.insert("character".to_string(), character);

        let sword = resources::load_model(
            "resources/sword.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "CUBE",
        )
        .await
        .unwrap();
        model_map.insert("sword".to_string(), sword);

        let sword_inventory = resources::load_model(
            "resources/sword.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "SQUARE",
        )
        .await
        .unwrap();
        model_map.insert("sword_inventory".to_string(), sword_inventory);

        let grass = resources::load_model(
            "resources/grass.jpg",
            &device,
            &queue,
            &texture_bind_group_layout,
            "CUBE",
        )
        .await
        .unwrap();
        model_map.insert("grass".to_string(), grass);

        let depth_texture = texture::DepthTexture::create_depth_texture(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            render_pipeline_ui,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_ui,
            camera_uniform_ui,
            camera_buffer_ui,
            camera_bind_group_ui,
            model_map,
            //obj_model: garfield,
            depth_texture,
            render_groups: Vec::new(),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::DepthTexture::create_depth_texture(&self.device, &self.config);
        }
    }

    // TODO Use this for handling input
    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    // Perhaps we want to use this instead of changing data in render method
    // pub fn update(&mut self) {
    // switch (input)
    // self.in
    // }

    pub fn render(
        &mut self,
        game_state: &GameState,
        ui_state: &UIState,
    ) -> Result<(), wgpu::SurfaceError> {
        // Update the camera in render with the game state data to build the new view
        // projection
        let player = "player".to_string();
        let camera = game_state.get_camera(player.clone()).unwrap();
        let rad_x = f32::to_radians(camera.rotation_x_degrees);
        let rad_y = f32::to_radians(camera.rotation_y_degrees);
        let player_position = game_state.get_position(player.clone()).unwrap();
        self.camera.eye = Point3 {
            x: player_position.x + camera.distance * rad_y.sin() * rad_x.cos(),
            y: player_position.y + camera.distance * rad_y.cos(),
            z: player_position.z + camera.distance * rad_y.sin() * rad_x.sin(),
        };
        self.camera.target = Point3 {
            x: player_position.x.clone(),
            y: 0.0, // player does not have an upwards direction yet
            z: player_position.y.clone(), // This can be confusing: our 2d world has x
                    // and y. in 3d the y is seen as vertical
        };

        self.camera_uniform.update_view_projection(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.create_render_groups(game_state);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                // depth_stencil_attachment: None,
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            self.render_groups.iter().for_each(|render_group| {
                render_pass.set_bind_group(
                    0,
                    &self
                        .model_map
                        .get(&render_group.model_id)
                        .unwrap()
                        .materials[0]
                        .bind_group,
                    &[],
                );
                render_pass.set_vertex_buffer(1, render_group.buffer.slice(..));
                render_pass.draw_mesh_instanced(
                    &self.model_map.get(&render_group.model_id).unwrap().meshes[0],
                    0..render_group.instance_count,
                );
            });

            drop(render_pass);
        }

        // UI
        if ui_state.inventory_open {
            self.set_camera_data_ui();
            let mut render_pass_ui = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass UI"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            render_pass_ui.set_pipeline(&self.render_pipeline_ui);
            render_pass_ui.set_bind_group(1, &self.camera_bind_group_ui, &[]);

            let inventory_instance = Self::create_inventory_instance(ui_state);
            let inventory_render_group = RenderGroup {
                buffer: Self::create_instance_buffer(&self.device, &vec![inventory_instance]),
                model_id: "sword_inventory".to_string(),
                instance_count: 1,
            };
            self.render_ui(&mut render_pass_ui, &inventory_render_group);

            let inventory_items = game_state.get_in_storages(&"player".to_string());
            //
            let render_groups = self.create_render_groups_ui(game_state, ui_state, inventory_items);
            render_groups.iter().for_each(|render_group| {
                self.render_ui(&mut render_pass_ui, render_group);
            });

            drop(render_pass_ui);
        }

        //use model::DrawModel;
        // let garfield = self.models.pop().unwrap();
        // let mesh = &garfield.meshes[0];
        // render_pass.draw_mesh_instanced(&garfield.meshes[0].clone(), 0..instances.len() as u32);
        //render_pass.draw_model_instanced(&self.obj_model, 0..instances.len() as u32);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn create_instance_buffer(
        device: &wgpu::Device,
        instance_group: &Vec<Instance>,
    ) -> wgpu::Buffer {
        let raw_instances = instance_group
            .iter()
            .map(|instance| Instance::to_raw(instance))
            .collect::<Vec<_>>();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"), // we could name this better
            contents: bytemuck::cast_slice(&raw_instances),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn to_instance(position: &Position) -> Instance {
        Instance {
            position: cgmath::Vector3 {
                x: position.x,
                y: position.z,
                z: position.y,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                // entity.size.x,
                // entity.size.z,
                // entity.size.y,
                1.0, 1.0, 1.0, 1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }
    }

    fn create_inventory_instance(ui_state: &UIState) -> Instance {
        Instance {
            position: cgmath::Vector3 {
                x: UIState::to_clip_space_x(ui_state.inventory_position_x),
                y: UIState::to_clip_space_y(ui_state.inventory_position_y),
                z: 0.0,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                UIState::to_scale_x(ui_state.inventory_width),
                UIState::to_scale_y(ui_state.inventory_height),
                1.0,
                1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }
    }

    fn create_inventory_item_instance(
        ui_state: &UIState,
        item: &InStorage,
        item_distance_x: f32,
        item_distance_y: f32,
        item_picture_scale_x: f32,
        item_picture_scale_y: f32,
    ) -> Instance {
        Instance {
            position: cgmath::Vector3 {
                x: UIState::to_clip_space_x(
                    ui_state.inventory_position_x + item.position_x as f32 * item_distance_x,
                ),
                y: UIState::to_clip_space_y(
                    ui_state.inventory_position_y + item.position_y as f32 * item_distance_y,
                ),
                z: 0.0,
            },
            scale: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(
                UIState::to_scale_x(item_picture_scale_x),
                UIState::to_scale_y(item_picture_scale_y),
                1.0,
                1.0,
            )),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }
    }

    fn create_render_groups(&mut self, game_state: &GameState) {
        let mut render_groups: Vec<RenderGroup> = Vec::new();
        game_state
            .entities
            .iter()
            .filter(|entity| {
                game_state
                    .graphics_3d_components
                    .contains_key(entity.as_str())
            })
            .group_by(|entity| {
                game_state
                    .get_graphics(entity.to_string())
                    .unwrap()
                    .model_id
                    .clone()
            })
            .into_iter()
            .for_each(|(model_id, group)| {
                let entity_group: Vec<&Entity> = group.collect();
                let instance_group: Vec<Instance> = entity_group
                    .into_iter()
                    .filter(|entity| game_state.get_position(entity.to_string()).is_some())
                    .map(|entity| {
                        Self::to_instance(game_state.get_position(entity.to_string()).unwrap())
                    })
                    .collect();
                let buffer = Self::create_instance_buffer(&self.device, &instance_group);
                let render_group = RenderGroup {
                    buffer,
                    model_id,
                    instance_count: instance_group.len() as u32,
                };
                render_groups.push(render_group);
            });
        self.render_groups = render_groups; // TODO reusing render_groups here, not too nice...
    }

    fn create_render_groups_ui(
        &self,
        game_state: &GameState,
        ui_state: &UIState,
        inventory_items: HashMap<&Entity, &InStorage>,
    ) -> Vec<RenderGroup> {
        let inventory = game_state.get_storage("player".to_string()).unwrap();
        let item_distance_x = ui_state.inventory_width / inventory.number_of_columns as f32;
        let item_distance_y = ui_state.inventory_height / inventory.number_of_rows as f32;
        let item_picture_scale_x = ui_state.inventory_width / inventory.number_of_columns as f32;
        let item_picture_scale_y = ui_state.inventory_height / inventory.number_of_rows as f32;

        let mut render_groups = Vec::new();
        inventory_items
            .iter()
            .group_by(|(entity, _)| {
                game_state
                    .get_graphics_inventory(entity.to_string())
                    .unwrap()
                    .model_id
                    .clone()
            })
            .into_iter()
            .for_each(|(model_id, group)| {
                let mut entity_group: Vec<&Entity> = Vec::new();
                let instance_group: Vec<Instance> = group
                    .into_iter()
                    .map(|(entity, in_storage)| {
                        entity_group.push(entity);
                        Self::create_inventory_item_instance(
                            ui_state,
                            in_storage,
                            item_distance_x,
                            item_distance_y,
                            item_picture_scale_x,
                            item_picture_scale_y,
                        )
                    })
                    .collect();
                let buffer = Self::create_instance_buffer(&self.device, &instance_group);
                let render_group = RenderGroup {
                    buffer,
                    model_id,
                    instance_count: instance_group.len() as u32,
                };
                render_groups.push(render_group);
            });
        render_groups
    }

    fn set_camera_data_ui(&mut self) {
        self.camera_ui.eye = Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        self.camera_ui.target = Point3 {
            x: 0.0,
            y: 0.0,  // player does not have an upwards direction yet
            z: -1.0, // This can be confusing: our 2d world has x
        }; // and y. in 3d the y is seen as vertical

        self.camera_ui.z_near = -1.0;
        self.camera_ui.z_far = 1.0;

        self.camera_uniform_ui
            .update_view_projection(&self.camera_ui);
        self.queue.write_buffer(
            &self.camera_buffer_ui,
            0,
            bytemuck::cast_slice(&[self.camera_uniform_ui]),
        );
    }

    fn render_ui(&'a self, render_pass: &mut RenderPass<'a>, render_group: &RenderGroup) {
        render_pass.set_bind_group(
                    0,
                    &self
                        .model_map
                        .get(&render_group.model_id)
                        .unwrap()
                        .materials[0]
                        .bind_group,
                    &[],
                );
                render_pass.set_vertex_buffer(1, render_group.buffer.slice(..));
                render_pass.draw_mesh_instanced(
                    &self.model_map.get(&render_group.model_id).unwrap().meshes[0],
                    0..render_group.instance_count,
                );
    }
}
