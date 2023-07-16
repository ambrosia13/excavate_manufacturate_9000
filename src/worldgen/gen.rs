use bevy::math::Vec3A;
use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;

use super::block::*;

fn hills(pos: Vec3A) -> Block {
    let noise = 10.0 * simplex_noise_2d(Vec2::new(pos.x * 0.01, pos.z * 0.01));

    if pos.y < noise {
        Block::Grass
    } else {
        Block::Air
    }
}

fn dirt(pos: Vec3A) -> Block {
    let current = hills(pos);
    let above = hills(pos + Vec3A::new(0.0, 1.0, 0.0));

    if above == Block::Grass {
        Block::Dirt
    } else {
        current
    }
}

fn water(pos: Vec3A) -> Block {
    let current = dirt(pos);

    if current == Block::Air && pos.y < -5.0 {
        Block::Water
    } else {
        current
    }
}

pub fn at_pos(pos: Vec3A) -> Block {
    let pos = pos.floor();

    water(pos)
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
