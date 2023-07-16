use bevy::prelude::*;
use bevy::math::Vec3A;

use super::block::*;

pub fn at_pos(pos: Vec3A) -> Block {
    let pos = pos.floor();

    if pos.y + 10.0 * noisy_bevy::simplex_noise_2d(Vec2::new(pos.x * 0.01, pos.z * 0.01)) < 0.0 {
        Block::Grass
    } else {
        Block::Air
    }
}

pub fn is_occluded(pos: Vec3A) -> bool {
    let pos = pos.floor();

    let mut occluded = true;

    occluded &= at_pos(pos + Vec3A::X).is_opaque();
    occluded &= at_pos(pos - Vec3A::X).is_opaque();

    occluded &= at_pos(pos + Vec3A::Y).is_opaque();
    occluded &= at_pos(pos - Vec3A::Y).is_opaque();

    occluded &= at_pos(pos + Vec3A::Z).is_opaque();
    occluded &= at_pos(pos - Vec3A::Z).is_opaque();

    occluded
}
