use crate::worldgen::block::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::Face;
use std::hash::{Hash, Hasher};

/// The size, in x, y, and z, of each chunk.
pub const CHUNK_SIZE: usize = 16;

/// The material to use for the chunk mesh.
pub const CHUNK_MATERIAL: StandardMaterial = StandardMaterial {
    base_color: Color::WHITE,
    base_color_texture: None,
    emissive: Color::BLACK,
    emissive_texture: None,
    perceptual_roughness: 1.0,
    metallic: 0.0,
    metallic_roughness_texture: None,
    reflectance: 1.0,
    normal_map_texture: None,
    flip_normal_map_y: false,
    occlusion_texture: None,
    double_sided: false,
    cull_mode: Some(Face::Back),
    unlit: false,
    fog_enabled: false,
    alpha_mode: AlphaMode::Opaque,
    depth_bias: 0.0,
    depth_map: None,
    parallax_depth_scale: 0.0,
    parallax_mapping_method: ParallaxMappingMethod::Occlusion,
    max_parallax_layer_count: 0.0,
};

pub struct Chunk {
    pub pos: IVec3,
    pub voxels: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    
    empty: bool,
}

// The following trait implementations are required for using a hash map
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl Eq for Chunk {}

impl Hash for Chunk {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
}

impl Chunk {
    pub fn empty(pos: IVec3) -> Self {
        Self {
            pos,
            voxels: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            empty: true,
        }
    }

    pub fn generate(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let world_pos: Vec3 =
                        self.pos.as_vec3() + Vec3::new(x as f32, y as f32, z as f32);

                    let block = crate::worldgen::gen::at_pos(world_pos.into());
                    self.voxels[x][y][z] = block;
                    self.empty &= block == Block::Air;
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn get_mesh(&self) -> Mesh {
        let mut builder = MeshBuilder::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if !self.voxels[x][y][z].is_opaque() {
                        continue;
                    }

                    let local_pos = Vec3::new(x as f32, y as f32, z as f32);

                    let pos_z = z.overflowing_add(1).0;
                    let neg_z = z.overflowing_sub(1).0;
                    let pos_y = y.overflowing_add(1).0;
                    let neg_y = y.overflowing_sub(1).0;
                    let pos_x = x.overflowing_add(1).0;
                    let neg_x = x.overflowing_sub(1).0;

                    let neighbor_pos_z = if (pos_z).clamp(0, CHUNK_SIZE - 1) == (pos_z) {
                        Some(self.voxels[x][y][pos_z])
                    } else {
                        None
                    };
                    let neighbor_neg_z = if (neg_z).clamp(0, CHUNK_SIZE - 1) == (neg_z) {
                        Some(self.voxels[x][y][neg_z])
                    } else {
                        None
                    };
                    let neighbor_pos_y = if (pos_y).clamp(0, CHUNK_SIZE - 1) == (pos_y) {
                        Some(self.voxels[x][pos_y][z])
                    } else {
                        None
                    };
                    let neighbor_neg_y = if (neg_y).clamp(0, CHUNK_SIZE - 1) == (neg_y) {
                        Some(self.voxels[x][neg_y][z])
                    } else {
                        None
                    };
                    let neighbor_pos_x = if (pos_x).clamp(0, CHUNK_SIZE - 1) == (pos_x) {
                        Some(self.voxels[pos_x][y][z])
                    } else {
                        None
                    };
                    let neighbor_neg_x = if (neg_x).clamp(0, CHUNK_SIZE - 1) == (neg_x) {
                        Some(self.voxels[neg_x][y][z])
                    } else {
                        None
                    };

                    let mut add_face_pos_z = true;
                    let mut add_face_neg_z = true;
                    let mut add_face_pos_y = true;
                    let mut add_face_neg_y = true;
                    let mut add_face_pos_x = true;
                    let mut add_face_neg_x = true;

                    if let Some(neighbor_pos_z) = neighbor_pos_z {
                        if neighbor_pos_z.is_opaque() {
                            add_face_pos_z = false;
                        }
                    }
                    if let Some(neighbor_neg_z) = neighbor_neg_z {
                        if neighbor_neg_z.is_opaque() {
                            add_face_neg_z = false;
                        }
                    }
                    if let Some(neighbor_pos_y) = neighbor_pos_y {
                        if neighbor_pos_y.is_opaque() {
                            add_face_pos_y = false;
                        }
                    }
                    if let Some(neighbor_neg_y) = neighbor_neg_y {
                        if neighbor_neg_y.is_opaque() {
                            add_face_neg_y = false;
                        }
                    }
                    if let Some(neighbor_pos_x) = neighbor_pos_x {
                        if neighbor_pos_x.is_opaque() {
                            add_face_pos_x = false;
                        }
                    }
                    if let Some(neighbor_neg_x) = neighbor_neg_x {
                        if neighbor_neg_x.is_opaque() {
                            add_face_neg_x = false;
                        }
                    }

                    if add_face_pos_z {
                        builder.add_face(
                            MeshBuilder::FACE_Z_FRONT,
                            MeshBuilder::NORMAL_Z_FRONT,
                            local_pos,
                        );
                    }
                    if add_face_neg_z {
                        builder.add_face(
                            MeshBuilder::FACE_Z_BACK,
                            MeshBuilder::NORMAL_Z_BACK,
                            local_pos,
                        );
                    }
                    if add_face_pos_y {
                        builder.add_face(
                            MeshBuilder::FACE_Y_FRONT,
                            MeshBuilder::NORMAL_Y_FRONT,
                            local_pos,
                        );
                    }
                    if add_face_neg_y {
                        builder.add_face(
                            MeshBuilder::FACE_Y_BACK,
                            MeshBuilder::NORMAL_Y_BACK,
                            local_pos,
                        );
                    }
                    if add_face_pos_x {
                        builder.add_face(
                            MeshBuilder::FACE_X_FRONT,
                            MeshBuilder::NORMAL_X_FRONT,
                            local_pos,
                        );
                    }
                    if add_face_neg_x {
                        builder.add_face(
                            MeshBuilder::FACE_X_BACK,
                            MeshBuilder::NORMAL_X_BACK,
                            local_pos,
                        );
                    }
                }
            }
        }

        builder.to_mesh()
    }
}

/// Utility object for building the chunk meshes
pub struct MeshBuilder {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub const FACE_Z_FRONT: [[f32; 3]; 4] = [
        [0.0, 0.0, 1.0], // Bottom left
        [0.0, 1.0, 1.0], // Top left
        [1.0, 0.0, 1.0], // Bottom right
        [1.0, 1.0, 1.0], // Top right
    ];
    pub const FACE_Z_BACK: [[f32; 3]; 4] = [
        [1.0, 0.0, 0.0], // Bottom right
        [1.0, 1.0, 0.0], // Top right
        [0.0, 0.0, 0.0], // Bottom left
        [0.0, 1.0, 0.0], // Top left
    ];
    pub const FACE_Y_FRONT: [[f32; 3]; 4] = [
        [0.0, 1.0, 1.0], // Front left
        [0.0, 1.0, 0.0], // Back left
        [1.0, 1.0, 1.0], // Front right
        [1.0, 1.0, 0.0], // Back right
    ];
    pub const FACE_Y_BACK: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0], // Front left
        [0.0, 0.0, 1.0], // Back left
        [1.0, 0.0, 0.0], // Front right
        [1.0, 0.0, 1.0], // Back right
    ];
    pub const FACE_X_FRONT: [[f32; 3]; 4] = [
        [1.0, 0.0, 1.0], // Front bottom
        [1.0, 1.0, 1.0], // Front top
        [1.0, 0.0, 0.0], // Back bottom
        [1.0, 1.0, 0.0], // Back top
    ];
    pub const FACE_X_BACK: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0], // Front bottom
        [0.0, 1.0, 0.0], // Front top
        [0.0, 0.0, 1.0], // Back bottom
        [0.0, 1.0, 1.0], // Back top
    ];

    pub const NORMAL_Z_FRONT: [[f32; 3]; 4] = [[0.0, 0.0, 1.0]; 4];
    pub const NORMAL_Z_BACK: [[f32; 3]; 4] = [[0.0, 0.0, -1.0]; 4];
    pub const NORMAL_Y_FRONT: [[f32; 3]; 4] = [[0.0, 1.0, 0.0]; 4];
    pub const NORMAL_Y_BACK: [[f32; 3]; 4] = [[0.0, -1.0, 0.0]; 4];
    pub const NORMAL_X_FRONT: [[f32; 3]; 4] = [[1.0, 0.0, 0.0]; 4];
    pub const NORMAL_X_BACK: [[f32; 3]; 4] = [[-1.0, 0.0, 0.0]; 4];

    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn get_face_indices(starting_index: u32) -> [u32; 6] {
        [
            starting_index,
            starting_index + 2,
            starting_index + 1,
            starting_index + 2,
            starting_index + 3,
            starting_index + 1,
        ]
    }

    /// Adds both the vertices and normals of the given face, provided the offset.
    pub fn add_face(&mut self, mut face: [[f32; 3]; 4], normals: [[f32; 3]; 4], offset: Vec3) {
        for i in 0..4 {
            for j in 0..3 {
                face[i][j] += offset[j];
            }
        }

        // The starting index of the vertices that are just going to be added to our Vec
        // E.g. if one face was added, the length would be 4, and the next face will start
        // from index 4. This is needed to provide the correct indices for each face.
        let starting_index = self.vertices.len();

        self.vertices.extend_from_slice(&face);
        self.normals.extend_from_slice(&normals);
        self.indices
            .extend_from_slice(&Self::get_face_indices(starting_index as u32));
    }

    pub fn to_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.set_indices(Some(Indices::U32(self.indices)));

        mesh
    }
}
