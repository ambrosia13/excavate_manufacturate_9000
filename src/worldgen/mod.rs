pub mod block;
pub mod chunk;
pub mod gen;

use crate::worldgen::chunk::timer::ChunkGenerationTimer;
use crate::worldgen::chunk::{ChunkQueue, GeneratedChunks, LoadedChunks};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkGenerationTimer(Timer::from_seconds(
            0.01,
            TimerMode::Repeating,
        )))
        .insert_resource(GeneratedChunks {
            map: Arc::new(Mutex::new(HashMap::new())),
        })
        .insert_resource(ChunkQueue(Arc::new(SegQueue::new())))
        .insert_resource(LoadedChunks {
            chunks: HashSet::new(),
        })
        .add_systems(Startup, chunk::loading::spawn_initial_chunks)
        .add_systems(
            Update,
            (
                chunk::timer::tick_chunk_generation_timer,
                chunk::generation::fill_chunk_queue,
                chunk::generation::generate_chunks_multithreaded,
                chunk::loading::load_generated_chunks,
                chunk::loading::unload_chunks,
            ),
        );
    }
}
