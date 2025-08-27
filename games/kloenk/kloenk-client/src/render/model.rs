use cgmath::Vector4;

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorTextureVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex for ColorTextureVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorTextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorDefinition {
    pub id: String,
    pub value: Vector4<f32>,
}

#[derive(Debug, Clone)]
pub struct TextureDefinition {
    pub id: String,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct ModelDefinition {
    pub id: String,
    pub primitives: Vec<PrimitiveDefinition>,
}

#[derive(Debug, Clone)]
pub struct PrimitiveDefinition {
    pub vertices_id: String,
    pub color_definition: ColorDefinition,
    pub texture_definition: Option<TextureDefinition>,
}
