/// Contains chunk generation logic; somewhat disconnected from Bevy (still uses Bevy types)
pub mod chunk_impl;
pub mod generation;
pub mod loading;
pub mod timer;

use crate::worldgen::block::Block;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::utils::{HashMap, HashSet};
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};

/// The size, in x, y, and z, of each chunk.
pub const CHUNK_SIZE: usize = 16;

// Should eventually be configurable
pub const VIEW_DISTANCE: i32 = 16;
pub const MAX_CHUNKS_PROCESSED_PER_ITER: usize = 16;

#[derive(Debug)]
pub struct Chunk {
    pub pos: IVec3,
    pub voxels: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],

    empty: bool,
}

/// Marker for whether this entity is terrain generated from the chunk meshing process.
#[derive(Component)]
pub struct ChunkedTerrain;

#[derive(Resource)]
pub struct GeneratedChunks {
    pub map: Arc<Mutex<HashMap<(i32, i32, i32), Chunk>>>,
}

#[derive(Resource)]
pub struct ChunkQueue(pub Arc<SegQueue<(i32, i32, i32)>>);

/// If a chunk position is in this list, then it is loaded.
#[derive(Resource)]
pub struct LoadedChunks {
    pub chunks: HashSet<(i32, i32, i32)>,
}

/// The material to use for the chunk mesh.
const CHUNK_MATERIAL: StandardMaterial = StandardMaterial {
    base_color: Color::WHITE,
    base_color_texture: None,
    emissive: Color::BLACK,
    emissive_texture: None,
    perceptual_roughness: 0.5,
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

pub fn chunk_material(asset_server: &Res<AssetServer>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/atlas.png")),
        ..CHUNK_MATERIAL
    }
}
