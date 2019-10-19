
use crate::basics::*;
use crate::draw::Drawable;
use crate::groups;

use specs::prelude::*;
use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{BodyPartHandle, ColliderDesc, DefaultBodyHandle, Ground, RigidBodyDesc};

#[derive(Component, Default)]
pub struct Block(Id, usize, usize);

// Can I just define a block as a normal item, but have it have a flag like "static" or something
// #[derive(Component)]
// struct Block {
//     kind: String,
//     x: usize,
//     y: usize,
// }

#[derive(Component)]
struct UnsupportedBlock {
    damage_per_tick: f32,
}

// here we generate the terrain
// and also handle "unsupported block" collapse

static BLOCK_SIZE: f32 = 0.4;

fn add_block(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    x: f32,
    y: f32,
    xi: usize,
    yi: usize,
) {
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(
        BLOCK_SIZE / 2.0,
        BLOCK_SIZE / 2.0,
    )));
    let builder = world.create_entity();
    let ground_collider = ColliderDesc::new(ground_shape)
        .user_data(builder.entity)
        .translation(Vector2::new(x, y))
        .collision_groups(groups::member_all_but_player())
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    builder
        .with(Collider(ground_collider))
        .with(Block(id("dirt"), xi, yi))
        .with(Drawable::Sprite {
            name: "dirt.png".into(),
            scale: 0.4,
        })
        .build();
}

pub fn make_blocks(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    _phys_w: f32,
    phys_h: f32,
) {
    let w = crate::WORLD_WIDTH / BLOCK_SIZE;

    for y in 1..40 {
        for i in 0..w as usize {
            add_block(
                world,
                physics_world,
                ground_handle,
                BLOCK_SIZE * i as f32, // * 1.01,
                phys_h - BLOCK_SIZE * 3.0 + BLOCK_SIZE * y as f32,
                i,
                y
            );
        }
    }
}