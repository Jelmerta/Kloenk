use crate::render::model::{
    ColorDefinition, ColorTextureVertex, ModelDefinition, PrimitiveDefinition, TextureDefinition,
};
use crate::render::primitive_vertices_manager::PrimitiveVertices;
use cgmath::Vector4;
use gltf::image::Source;
use gltf::mesh::util::ReadIndices;
use gltf::Gltf;
use hydrox::load_binary;

// // or like AssetLoadTask / AssetLoadCommand
// // Maybe also in charge of unloading? Just making sure the current state of the world is loaded correctly
pub struct ModelLoader {}

impl ModelLoader {
    pub fn load_colored_square_model(name: String, color: Vector4<f32>) -> ModelDefinition {
        ModelDefinition {
            id: name.clone() + "_square",
            primitives: vec![PrimitiveDefinition {
                vertices_id: "SQUARE".to_string(),
                color_definition: ColorDefinition {
                    id: name,
                    value: color,
                },
                texture_definition: None,
            }],
        }
    }

    // Assume for now one model for each gltf
    // Maybe at some point we just want to preload a scene?
    pub async fn preload_gltf(model_path: &str) -> Vec<ModelDefinition> {
        // let data = include_bytes!("../../assets/gozer.gltf"); // TODO just hardcoded the one model into memory as we don't want to introduce async here probably
        let data = load_binary(model_path)
            .await
            .unwrap_or_else(|_| panic!("Path {} could not be found", model_path));
        let gltf = Gltf::from_slice(data.as_slice())
            .unwrap_or_else(|_| panic!("Failed to load gltf model {}", model_path));

        let mut model_definitions = Vec::new();
        for mesh in gltf.meshes() {
            let mut primitives = Vec::new();
            mesh.primitives().for_each(|primitive| {
                let metal = primitive.material().pbr_metallic_roughness();
                let primitive_color = metal.base_color_factor();

                let texture = metal.metallic_roughness_texture();
                let texture_uri = texture.map(|info| match info.texture().source().source() {
                    Source::View { .. } => {
                        panic!("Only supports URI")
                    }
                    Source::Uri { uri, .. } => uri.to_string(),
                });

                let primitive_definition = PrimitiveDefinition {
                    vertices_id: model_path.to_string(), // TODO primitive level
                    color_definition: ColorDefinition {
                        // TODO Note id needs to be on primitive level cause different primitives can have different colours
                        id: mesh.name().unwrap().to_string(), // TODO maybe have some kind of unique id for colors, maybe readable hexvalue? idk.
                        value: primitive_color.into(),
                    },
                    texture_definition: texture_uri.map(|uri| TextureDefinition {
                        id: uri.clone(),
                        file_name: uri,
                    }),
                };
                primitives.push(primitive_definition);
            });
            let model_definition = ModelDefinition {
                id: mesh
                    .name()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| "no name".to_string()), // todo panic?
                primitives,
            };
            model_definitions.push(model_definition);
        }
        model_definitions
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

                // let material = primitive.material();
                // let primitive_color = material.pbr_metallic_roughness().base_color_factor();
                // TODO load color?

                let vertices = if let Some(vertex_attribute) = reader.read_positions() {
                    let mut vertices = Vec::new();

                    for (index, vertex) in vertex_attribute.enumerate() {
                        vertices.push(ColorTextureVertex {
                            position: vertex,
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
    pub fn make_preload_model(
        model_name: String,
        model_to_load: &str,
        material_file_uri: &str,
    ) -> ModelDefinition {
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

        ModelDefinition {
            id: model_name,
            primitives: vec![PrimitiveDefinition {
                vertices_id: model_to_load.to_string(),
                color_definition: ColorDefinition {
                    id: "white".to_string(),
                    value: Vector4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        w: 1.0,
                    },
                },
                texture_definition: Some(TextureDefinition {
                    id: material_file_uri.to_string(),
                    file_name: material_file_uri.to_string(),
                }),
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
