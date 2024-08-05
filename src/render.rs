use std::{iter, mem, primitive};

// use anyhow::*;
use cgmath::{prelude::*, Point3};
use gltf::iter::Meshes;
use gltf::mesh::util::indices;
use gltf::texture as gltf_texture;
use gltf::Gltf;
use wasm_bindgen::prelude::*;
use web_sys::console;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::camera::{self, Camera};
use crate::game_state::{self, GameState, Position};
use crate::model::Vertex;
use crate::model::{self, TexVertex};
use crate::texture;
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
        use cgmath::SquareMatrix; // ?
        Self {
            // view_proj: cgmath::Matrix4::identity().into(),
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    depth_texture: texture::Texture,
    instance_buffer: wgpu::Buffer,
    inv_instance_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
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

const TRIANGLE: &[model::ModelVertex] = &[
    model::ModelVertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    model::ModelVertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    model::ModelVertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

const PENTAGON: &[model::ModelVertex] = &[
    model::ModelVertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    model::ModelVertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // B
    model::ModelVertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // C
    model::ModelVertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // D
    model::ModelVertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.5, 0.5],
    }, // E
];

const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

// Hm, does using indices lose me the ability to color sides of the vertices differently? Great if you use textures, but otherwise kinda sucks. Would not be able to draw something like a rubiks cube. Onless we can define these in the index buffer instead i guess?
const CUBE: &[model::ModelVertex] = &[
    // Top ccw as seen from top
    model::ModelVertex {
        position: [0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.0],
    }, // Red
    model::ModelVertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 0.5, 0.0],
    }, // Green
    model::ModelVertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.5, 0.0],
    }, // Yellow
    model::ModelVertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.5],
    }, // Purple
    // Bottom ccw as seen from top
    model::ModelVertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 0.0, 0.5],
    }, // Blue
    model::ModelVertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 0.5, 0.5],
    }, // Cyan
    model::ModelVertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.0, 0.0],
    }, // Black
    model::ModelVertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.5, 0.5, 0.5],
    }, // White
];

const CUBE_TEX: &[model::TexVertex] = &[
    // Top ccw as seen from top
    model::TexVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    }, // Red
    model::TexVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // Green
    model::TexVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // Yellow
    model::TexVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Purple
    // Bottom ccw as seen from top
    model::TexVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Blue
    model::TexVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // Cyan
    model::TexVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // Black
    model::TexVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
    }, // White
];

const CUBE_INDICES: &[u16] = &[
    // Top
    0, 1, 2, 0, 2, 3, // Bottom
    4, 7, 6, 4, 6, 5, // Left
    0, 3, 7, 0, 7, 4, // Right
    1, 6, 2, 1, 5, 6, // Front
    0, 4, 5, 0, 5, 1, // Back
    2, 6, 7, 2, 7, 3,
];

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
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
        };
        surface.configure(&device, &config);

        // ----- texture stuff
        // let diffuse_bytes = include_bytes!("../resources/perocaca.jpg");
        let diffuse_bytes = include_bytes!("../resources/perocaca.jpg");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            //
            // width: 256,
            // height: 256,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });
        // log::warn!("{:?}", diffuse_rgba);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

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

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
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
                format: texture::Texture::DEPTH_FORMAT,
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
            }),
            multiview: None,
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
            }),
            multiview: None,
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
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(CUBE_TEX),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = CUBE_INDICES.len() as u32;

        let depth_texture = texture::Texture::create_depth_texture(&device, &config);

        // let cube_instance_character = Instance {
        //     position: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        //     rotation: cgmath::Quaternion::from_axisG_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
        // };
        //
        // // let cube_instance_enemy = Instance {
        // //     position: cgmath::Vector3 { x: 2.0, y: 0.0, z: 2.0 },
        // //     rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(90.0)),
        // // };
        // // let cube_instance_enemy_2 = Instance {
        // //     position: cgmath::Vector3 { x: -2.0, y: 0.0, z: 1.5 },
        // //     rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(90.0)),
        // // };
        // // let cube_instance_enemy_3 = Instance {
        // //     position: cgmath::Vector3 { x: -1.0, y: -1.0, z: -1.0 },
        // //     rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(180.0)),
        // // };
        // // let cube_instance_enemy_4 = Instance {
        // //     position: cgmath::Vector3 { x: 1.0, y: 1.0, z: -1.0 },
        // //     rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(45.0)),
        // // };
        // let mut instances = Vec::new();
        // instances.push(cube_instance_character);
        // // instances.push(cube_instance_enemy);
        // // instances.push(cube_instance_enemy_2);
        // // instances.push(cube_instance_enemy_3);
        // // instances.push(cube_instance_enemy_4);
        //
        // let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // Tmp assign a buffer. Should be removed.
        let instance_data = Vec::new().iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let inv_instance_data = Vec::new().iter().map(Instance::to_raw).collect::<Vec<_>>();
        let inv_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Inventory Instance Buffer"),
            contents: bytemuck::cast_slice(&inv_instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

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
            vertex_buffer,
            index_buffer,
            num_indices,
            //obj_model: garfield,
            depth_texture,
            instance_buffer,
            inv_instance_buffer,
            diffuse_bind_group,
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
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        // switch (input)
        // self.in
    }

    pub fn render(&mut self, game_state: &GameState) -> Result<(), wgpu::SurfaceError> {
        // Update the camera in render with the game state data to build the new view
        // projection

        // Use 45 degrees for isometric view.
        let angle = std::f32::consts::FRAC_PI_4;
        let rad_x = f32::to_radians(game_state.camera_rotation_x_degrees);
        let rad_y = f32::to_radians(game_state.camera_rotation_y_degrees);
        self.camera.eye = Point3 {
            x: game_state.player.position.x
                + game_state.camera_distance * rad_y.sin() * rad_x.cos(),
            y: game_state.player.position.y + game_state.camera_distance * rad_y.cos(),
            z: game_state.player.position.z
                + game_state.camera_distance * rad_y.sin() * rad_x.sin(),
        };
        self.camera.target = Point3 {
            x: game_state.player.position.x.clone(),
            y: 0.0, // player does not have an upwards direction yet
            z: game_state.player.position.y.clone(), // This can be confusing: our 2d world has x
                    // and y. in 3d the y is seen as vertical
        };

        self.camera_uniform.update_view_projection(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // Set buffer data
            let mut instances = Vec::new();
            for entity in game_state.get_entities().into_iter() {
                let position = entity.get_position();
                let instance = Instance {
                    position: cgmath::Vector3 {
                        x: position.get_x(),
                        y: 0.0,
                        z: position.get_y(),
                    },
                    rotation: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                };
                instances.push(instance);
            }

            let player_position = game_state.player.get_position();
            let player_instance = Instance {
                position: cgmath::Vector3 {
                    x: player_position.get_x(),
                    y: 0.0,
                    z: player_position.get_y(),
                },
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
            };
            instances.push(player_instance);

            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            let instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            self.instance_buffer = instance_buffer; // This gets around a borrow check error... Not sure what the best way to do this is...

            // Add buffers to render pass
            // render_pass.set_vertex_buffer(0, garfield..vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..instances.len() as _);
            drop(render_pass);
        }
        // UI
        if (game_state.inventory_toggled) {
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
            render_pass_ui.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass_ui.set_bind_group(1, &self.camera_bind_group_ui, &[]);

            log::warn!("{:?}", self.camera.eye);
            log::warn!(
                "{}, {}",
                game_state.player.position.x,
                game_state.player.position.y
            );
            self.camera_ui.eye = Point3 {
                x: 0.0,
                y: 0.0,
                z: 100.0,
            };

            self.camera_ui.target = Point3 {
                x: 0.0,
                y: 0.0, // player does not have an upwards direction yet
                z: 0.0, // This can be confusing: our 2d world has x
                        // and y. in 3d the y is seen as vertical
            };

            self.camera_uniform_ui
                .update_view_projection(&self.camera_ui);
            self.queue.write_buffer(
                &self.camera_buffer_ui,
                0,
                bytemuck::cast_slice(&[self.camera_uniform_ui]),
            );

            let mut inv_instance_data = Vec::new();

            let inventory_instance = Instance {
                position: cgmath::Vector3 {
                    x: game_state.inventory_position.get_x(),
                    y: game_state.inventory_position.get_y(),
                    z: 0.0,
                },
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
            };
            inv_instance_data.push(Instance::to_raw(&inventory_instance));

            let mut instances = 1;
            if (game_state.inventory_has_item) {
                let inventory_item_instance = Instance {
                    position: cgmath::Vector3 {
                        x: game_state.inventory_position.get_x() + 0.5,
                        y: game_state.inventory_position.get_y() - 0.5,
                        z: -60.0,
                    },
                    rotation: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                };
                inv_instance_data.push(Instance::to_raw(&inventory_item_instance));
                instances = 2;
            }

            let inv_instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Inventory Instance Buffer"),
                        contents: bytemuck::cast_slice(&inv_instance_data),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            self.inv_instance_buffer = inv_instance_buffer; // This gets around a borrow check error... Not sure what the best way to do this is...
                                                            //

            render_pass_ui.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass_ui.set_vertex_buffer(1, self.inv_instance_buffer.slice(..));
            render_pass_ui.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass_ui.draw_indexed(0..self.num_indices, 0, 0..instances as _);
            drop(render_pass_ui);
        }

        // if (
        // instances.push(inventory_items_instance);
        //

        //use model::DrawModel;
        // let garfield = self.models.pop().unwrap();
        // let mesh = &garfield.meshes[0];
        // render_pass.draw_mesh_instanced(&garfield.meshes[0].clone(), 0..instances.len() as u32);
        //render_pass.draw_model_instanced(&self.obj_model, 0..instances.len() as u32);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
