use crate::render::camera_manager::CameraManager;
use crate::render::instance::InstanceRaw;
use crate::render::material_manager::MaterialManager;
use crate::render::model::Vertex;
use crate::render::{model, texture};
use std::collections::HashMap;
use wgpu::{Device, PipelineCompilationOptions, RenderPipeline, SurfaceConfiguration};

pub struct RenderContext {
    // pub shader: ShaderModule,
    pub render_pipeline: RenderPipeline,
}

pub struct RenderContextManager {
    pub render_contexts: HashMap<String, RenderContext>,
}

impl RenderContextManager {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_manager: &CameraManager,
        material_manager: &MaterialManager,
    ) -> Self {
        let mut render_contexts = HashMap::new();
        render_contexts.insert(
            "textured".to_string(),
            Self::setup_textured_context(device, config, camera_manager, material_manager),
        );
        render_contexts.insert(
            "colored".to_string(),
            Self::setup_colored_context(device, config, camera_manager),
        );
        RenderContextManager { render_contexts }
    }

    fn setup_colored_context(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_manager: &CameraManager,
    ) -> RenderContext {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Color Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/color_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_manager.bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model::ColoredVertex::layout(), InstanceRaw::layout()],
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
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        RenderContext { render_pipeline }
    }

    fn setup_textured_context(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_manager: &CameraManager,
        material_manager: &MaterialManager,
    ) -> RenderContext {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/texture_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &material_manager.bind_group_layout,
                    &camera_manager.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model::TexVertex::layout(), InstanceRaw::layout()],
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
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(), // required for web srgb otherwise shows bland colours could add this only if wasm32
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),  // Seems to be necessary for webp to correctly show transparency? on desktop i did not need count 2, on web getting error without count 2. not fully understanding this yet. i suppose if we do not use webp we do not need this. could make specific pipeline for webp who knows
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });
        RenderContext { render_pipeline }
    }
}
