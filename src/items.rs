use crate::basics::{Body, Collider, PhysicsWorld};
use crate::{draw};
use na::{Point2, Vector2};

use ncollide2d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, RigidBodyDesc,
    RigidBody
};


pub fn create_rock(
    physics: &mut PhysicsWorld<f32>,
    entities: &specs::Entities,
    bodies: &mut specs::storage::WriteStorage<Body>,
    colliders: &mut specs::storage::WriteStorage<Collider>,
    drawables: &mut specs::storage::WriteStorage<draw::Drawable>,
    pos: Vector2<f32>,
    size: f32,
    sprite: String
) {

    // things that make it a ball
    let ball_shape = Ball::new(size);
    let drawable = crate::draw::Drawable::Sprite {
        name: sprite,
        scale: 5.0 * size,
    };
    let rb = RigidBodyDesc::new()
        .translation(pos)
        .rotation(rand::random::<f32>() * std::f32::consts::PI * 2.0)
        // .set_velocity(vel)
        .build();
    let rb_handle = physics.bodies.insert(rb);

    // Build the collider.
    let mut material = nphysics2d::material::BasicMaterial::new(0.1, 0.5);
    material.restitution_combine_mode = nphysics2d::material::MaterialCombineMode::Multiply;
    let mh = nphysics2d::material::MaterialHandle::new(material);

    let entity = entities.create();

    let co = ColliderDesc::new(ShapeHandle::new(ball_shape))
        .density(1.0)
        .user_data(entity)
        .material(mh.clone())
        .ccd_enabled(true)
        .collision_groups(crate::groups::default())
        .build(BodyPartHandle(rb_handle, 0));
    let co_handle = physics.colliders.insert(co);

    bodies.insert(entity, Body(rb_handle)).unwrap();
    colliders.insert(entity, Collider(co_handle)).unwrap();
    drawables.insert(entity, drawable).unwrap();
}
