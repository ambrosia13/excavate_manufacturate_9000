use bevy::prelude::*;

/// Used to delay the generation of chunks so it doesn't happen constantly
#[derive(Resource)]
pub struct ChunkGenerationTimer(pub Timer);

pub fn tick_chunk_generation_timer(mut timer: ResMut<ChunkGenerationTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}
