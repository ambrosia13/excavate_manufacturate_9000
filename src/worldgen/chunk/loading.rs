use crate::camera::PlayerCamera;
use crate::worldgen::chunk::{
    chunk_material, Chunk, ChunkedTerrain, GeneratedChunks, LoadedChunks, CHUNK_SIZE,
    MAX_CHUNKS_PROCESSED_PER_ITER, VIEW_DISTANCE,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_initial_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    generated_chunks: ResMut<GeneratedChunks>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    asset_server: Res<AssetServer>,
) {
    // Initial render distance.
    // This doesn't need to be high, because other chunks will be loaded during runtime;
    // these chunks are the chunks that are loaded before the game starts.
    let initial_view_distance = 2;

    let mut chunk_map = generated_chunks.map.lock().unwrap();

    for x in -initial_view_distance..=initial_view_distance {
        for y in -initial_view_distance..=initial_view_distance {
            for z in -initial_view_distance..=initial_view_distance {
                let pos = IVec3::new(x, y, z);

                let mut chunk = Chunk::empty(pos * CHUNK_SIZE as i32);
                chunk.generate();

                // Skip the whole mesh-making-process for empty chunks
                if chunk.is_empty() {
                    chunk_map.insert((pos.x, pos.y, pos.z), chunk);
                    loaded_chunks.chunks.insert((pos.x, pos.y, pos.z));

                    continue;
                }

                let mesh = chunk.get_mesh();

                commands
                    .spawn((
                        // Physics component
                        (
                            RigidBody::Fixed,
                            Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh)
                                .unwrap(),
                        ),
                        // Geometry component
                        PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(chunk_material(&asset_server)),
                            transform: Transform::from_translation(chunk.pos.as_vec3()),
                            ..default()
                        },
                    ))
                    .insert(ChunkedTerrain);

                chunk_map.insert((pos.x, pos.y, pos.z), chunk);
                loaded_chunks.chunks.insert((pos.x, pos.y, pos.z));
            }
        }
    }
}

pub fn load_generated_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    generated_chunks: Res<GeneratedChunks>,
    asset_server: Res<AssetServer>,

    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    let mut map = generated_chunks.map.lock().unwrap();

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;
    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let mut chunks_spawned = 0;

    for (chunk_pos, chunk) in map.iter_mut() {
        if chunks_spawned == MAX_CHUNKS_PROCESSED_PER_ITER {
            return;
        }

        let offsetted_chunk_pos = camera_chunk_pos - IVec3::from(*chunk_pos);

        // Only load the chunk if it's in the view distance
        // This is required to prevent this system from trying to spawn
        // chunks that the despawn system has just destroyed.
        if !((-VIEW_DISTANCE.0..=VIEW_DISTANCE.0).contains(&offsetted_chunk_pos.x)
            && (-VIEW_DISTANCE.1..=VIEW_DISTANCE.1).contains(&offsetted_chunk_pos.y)
            && (-VIEW_DISTANCE.2..=VIEW_DISTANCE.2).contains(&offsetted_chunk_pos.z))
        {
            continue;
        }

        // If the chunk is empty or already loaded, don't bother loading it
        if chunk.is_empty() || loaded_chunks.chunks.contains(chunk_pos) {
            continue;
        }

        let mesh = chunk.get_mesh();

        commands
            .spawn((
                // Physics component
                (
                    RigidBody::Fixed,
                    Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap(),
                ),
                // Geometry component
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(chunk_material(&asset_server)),
                    transform: Transform::from_translation(chunk.pos.as_vec3()),
                    ..default()
                },
            ))
            .insert(ChunkedTerrain);

        loaded_chunks.chunks.insert(*chunk_pos);

        chunks_spawned += 1;
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunks_query: Query<(Entity, &Transform), With<ChunkedTerrain>>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;
    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let mut chunks_despawned = 0;

    for (entity, transform) in chunks_query.iter() {
        if chunks_despawned == MAX_CHUNKS_PROCESSED_PER_ITER {
            return;
        }

        let chunk_position = transform.translation.as_ivec3() / CHUNK_SIZE as i32;

        let offsetted_chunk_pos = chunk_position - camera_chunk_pos;

        // If the chunk isn't in the view distance, despawn it
        if !((-VIEW_DISTANCE.0..=VIEW_DISTANCE.0).contains(&offsetted_chunk_pos.x)
            && (-VIEW_DISTANCE.1..=VIEW_DISTANCE.1).contains(&offsetted_chunk_pos.y)
            && (-VIEW_DISTANCE.2..=VIEW_DISTANCE.2).contains(&offsetted_chunk_pos.z))
        {
            commands.entity(entity).despawn();

            loaded_chunks
                .chunks
                .remove::<(i32, i32, i32)>(&chunk_position.into());

            chunks_despawned += 1;
        }
    }
}
