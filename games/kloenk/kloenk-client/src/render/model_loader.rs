use crate::render::model::MaterialToLoad::{Color, Texture};
use crate::render::model::{
    ColorTextureVertex, ModelToLoad, PrimitiveToLoad,
};
use crate::render::primitive_vertices_manager::PrimitiveVertices;
use cgmath::Vector3;
use gltf::image::Source;
use gltf::mesh::util::ReadIndices;
use gltf::Gltf;
use hydrox::load_binary;

const CUBE_TEX: &[ColorTextureVertex] = &[
    // Top ccw as seen from top
    ColorTextureVertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    ColorTextureVertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    ColorTextureVertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    // Bottom ccw as seen from top
    ColorTextureVertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    ColorTextureVertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    ColorTextureVertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Bottom
    4, 7, 6, 4, 6, 5, // Left
    0, 3, 7, 0, 7, 4, // Right
    1, 6, 2, 1, 5, 6, // Front
    0, 4, 5, 0, 5, 1, // Back
    2, 6, 7, 2, 7, 3,
];

const SQUARE_TEX: &[ColorTextureVertex] = &[
    ColorTextureVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    ColorTextureVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    ColorTextureVertex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

const SQUARE_INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];

pub struct ModelLoader {}

impl ModelLoader {
    pub fn load_colored_square_model(name: String, color: Vector3<f32>) -> ModelToLoad {
        // let model = load_colored_square(color);
        // let indices = SQUARE_INDICES;

        // let meshes = build_colored_meshes(&&model[..], &indices, color);

        ModelToLoad {
            name: name.clone(),
            primitives_to_load: vec![PrimitiveToLoad {
                vertices_to_load: "square".to_string(),
                material_to_load: Color { name, rgb: color },
            }],
        }
    }

    // pub fn load_textured_square(file_name: String) -> ModelToLoad {
    //     ModelToLoad {
    //         name: file_name.clone(),
    //         primitives_to_load: vec![PrimitiveToLoad {
    //             vertices_to_load: "square".to_string(),
    //             material_to_load: Texture { file_name },
    //         }],
    //     }
    // }
    //
    // pub fn load_textured_cube(file_name: String) -> ModelToLoad {
    //     ModelToLoad {
    //         name: file_name.clone(),
    //         primitives_to_load: vec![PrimitiveToLoad {
    //             vertices_to_load: "cube".to_string(),
    //             material_to_load: Texture { file_name },
    //         }],
    //     }
    // }

    fn build_colored_1x1_texture() {}

    // TODO should we save the gltf data immediately as well...? dont want to read it twice
    // TODO load models without texture as well... if there is no image? primitives dont need image necessarily i suppose?
    pub async fn preload_gltf(model_path: &str) -> Vec<ModelToLoad> {
        log::error!("Preloading GLTF");
        // Used before renderer loaded to set required data
        let data = load_binary(model_path)
            .await
            .unwrap_or_else(|_| panic!("Path {} could not be found", model_path));
        let gltf = Gltf::from_slice(data.as_slice())
            .unwrap_or_else(|_| panic!("Failed to load gltf model {}", model_path));

        // TODO note we can have multiple primitives, each with different materials
        let mut models = vec![];
        gltf.images().for_each(|image| match image.source() {
            Source::Uri { uri, mime_type: _ } => {
                log::error!("Loading image {}", uri);
                let material_file_uri = uri.to_string();
                models.push(ModelToLoad {
                    name: material_file_uri.clone(), // TODO name for different materials
                    primitives_to_load: vec![PrimitiveToLoad {
                        vertices_to_load: model_path.to_string(),
                        material_to_load: Texture {
                            file_name: material_file_uri,
                        },
                    }],
                });
            }
            Source::View { .. } => {
                panic!("Views not supported");
            }
        });

        models
    }

    // TODO maybe make an intermediate step and then load everything into gpu buffers with intermediate object?
    pub async fn load_gltf(model_path: &str) -> Vec<PrimitiveVertices> {
        let data = load_binary(model_path)
            .await
            .unwrap_or_else(|_| panic!("Path {} could not be found", model_path));
        let gltf = Gltf::from_slice(data.as_slice())
            .unwrap_or_else(|_| panic!("Failed to load gltf model {}", model_path));

        let mut buffer_data = Vec::new();
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => { // glb (not used (yet?))
                }

                // Gltf + bin TODO is this async done after the model without dds / bin is loaded? should not be blocking for using a backup
                gltf::buffer::Source::Uri(uri) => {
                    let bin = load_binary(uri).await.unwrap();
                    buffer_data.push(bin);
                }
            }
        }

        let mut primitive_vertices = Vec::new();
        for mesh in gltf.meshes() {
            mesh.primitives().for_each(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let tex_coords = reader
                    .read_tex_coords(0)
                    .map(|read_tex_coords| read_tex_coords.into_f32().collect::<Vec<[f32; 2]>>())
                    .unwrap();

                let material = primitive.material();
                let primitive_color = material.pbr_metallic_roughness().base_color_factor();

                let vertices = if let Some(vertex_attribute) = reader.read_positions() {
                    let mut vertices = Vec::new();

                    for (index, vertex) in vertex_attribute.enumerate() {
                        vertices.push(ColorTextureVertex {
                            position: vertex,
                            color: primitive_color,
                            tex_coords: *tex_coords.get(index).unwrap(),
                        })
                    }
                    vertices
                } else {
                    // TODO probably panic?
                    Vec::new()
                };

                let indices = if let Some(read_indices) = reader.read_indices() {
                    let mut indices = Vec::new();
                    match read_indices {
                        ReadIndices::U8(iter) => {
                            iter.for_each(|index| indices.push(index as u16));
                        }
                        ReadIndices::U16(iter) => {
                            iter.for_each(|index| indices.push(index));
                        }
                        ReadIndices::U32(iter) => {
                            iter.for_each(|index| indices.push(index as u16));
                        }
                    }
                    indices
                } else {
                    Vec::new()
                };

                // primitive.mappings()

                // // TODO Not used... Vertex buffer is already set containing the colors
                // let vertex_type = if primitive.material().index().is_some() {
                //     let color: [f32; 3];
                //     color.copy_from_slice(&primitive.material().pbr_metallic_roughness().base_color_factor()[0..3]);
                //     VertexType::Color {
                //         color,
                //         // color: primitive.material().emissive_factor(),
                //     }
                // } else {
                //     // No material found for the primitive, so just default to a color
                //     VertexType::Color {
                //         color: Vector3::new(0.7, 0.7, 0.7).into(),
                //     }
                // };

                primitive_vertices.push(PrimitiveVertices {
                    // primitive_vertices_id: "".to_string(),
                    // vertex_type,
                    name: model_path.to_string(),
                    vertices,
                    indices,
                    // material_id: "".to_string(),
                });

                // TODO load into vertices / material managers
            })
        }
        primitive_vertices
    }

    // #[allow(clippy::cast_possible_truncation)]
    pub fn make_preload_model(model_name: String, model_to_load: &str, material_file_uri: &str) -> ModelToLoad {
        // let model: &[TexVertex];
        // let indices: &[u16];
        // if model_to_load.eq("CUBE") {
        //     model = CUBE_TEX;
        //     indices = CUBE_INDICES;
        // } else {
        //     model = SQUARE_TEX;
        //     indices = SQUARE_INDICES;
        // }

        // Also add this to material
        // name: file_name.to_string(),
        // diffuse_texture,

        ModelToLoad {
            name: model_name,
            primitives_to_load: vec![PrimitiveToLoad {
                vertices_to_load: model_to_load.to_string(),
                material_to_load: Texture {
                    file_name: material_file_uri.to_string(),
                },
            }],
        }
    }
    //
    // // TODO wondering if we can't reuse same 3d meshes with different material with same buffers
    // fn build_textured_meshes(
    //     mesh_material_id: &str,
    // ) -> Vec<Primitive> {
    //     let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Vertex Buffer"),
    //         contents: bytemuck::cast_slice(model),
    //         usage: wgpu::BufferUsages::VERTEX,
    //     });
    //
    //     let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Index Buffer"),
    //         contents: bytemuck::cast_slice(indices),
    //         usage: wgpu::BufferUsages::INDEX,
    //     });
    //     let num_indices = indices.len() as u32;
    //
    //     let meshes = vec![Primitive {
    //         primitive_vertices_id: mesh_vertices_id.to_string(), // instead of vertex/index buffer. those can be preloaded? well they require device. maybe in constructor. hmm but we dynamically load
    //         vertex_type: VertexType::Material {
    //             material_id: mesh_material_id.to_string(), // Currently we set all meshes of a model to the same material
    //         },
    //         // vertex_buffer,
    //         // index_buffer,
    //         // num_elements: num_indices,
    //     }];
    //     meshes
    // }

    // fn build_colored_meshes(
    //     color: Vector3<f32>,
    // ) -> Vec<Primitive> {
    //     let meshes = vec![Primitive {
    //         primitive_vertices_id: mesh_vertices_id,
    //         vertex_type: VertexType::Color {
    //             color: color.into(), // Currently we set all meshes of a model to the same color
    //         },
    //     }];
    //     meshes
    // }
}
