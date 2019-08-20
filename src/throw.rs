use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyPartHandle, ColliderDesc, DefaultColliderHandle, RigidBodyDesc};

use crate::basics::*;
use crate::draw::Drawable;

#[derive(Component)]
pub struct Thrown(DefaultColliderHandle, usize);

pub struct ThrownSys;

impl<'a> System<'a> for ThrownSys {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        WriteStorage<'a, Thrown>,
        WriteStorage<'a, Collider>,
    );

    fn run(&mut self, (entities, mut physics_world, mut sensors, colliders): Self::SystemData) {
        let mut to_remove = vec![];
        for (entity, Thrown(parent_collider, parent_group), collider) in
            (&entities, &mut sensors, &colliders).join()
        {
            if physics_world
                .geom
                .proximity_pair(&physics_world.colliders, collider.0, *parent_collider, true)
                .is_none()
            {
                to_remove.push(entity);
                if let Some(collider) = physics_world.collider_mut(collider.0) {
                    let mut groups = *collider.collision_groups();
                    groups.modify_blacklist(*parent_group, false);
                    collider.set_collision_groups(groups);
                }
            }
        }
        for entity in to_remove {
            sensors.remove(entity);
        }
    }
}

#[derive(Component)]
pub struct ArrowLauncher(pub Option<Vector2<f32>>, pub DefaultColliderHandle);

pub struct ArrowSys;

static MIN_THROW: f32 = 10.0;

impl<'a> System<'a> for ArrowSys {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, raylib::RaylibHandle>,
        Read<'a, crate::ZoomCamera>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        WriteStorage<'a, ArrowLauncher>,
        WriteStorage<'a, crate::skeletons::component::Skeleton>,
        WriteStorage<'a, Thrown>,
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Body>,
        WriteStorage<'a, Drawable>,
    );

    fn run(
        &mut self,
        (
            entities,
            rl,
            zoom_camera,
            mut physics_world,
            mut arrow,
            mut skeletons,
            mut sensors,
            mut colliders,
            mut bodies,
            mut drawables,
        ): Self::SystemData,
    ) {
        if let Some((mut arrow, mut skeleton, collider_entity)) =
            (&mut arrow, &mut skeletons, &colliders).join().next()
        {
            if rl.is_mouse_button_pressed(raylib::consts::MouseButton::MOUSE_LEFT_BUTTON) {
                let vec = rl.get_mouse_position();
                arrow.0 = Some(Vector2::new(vec.x, vec.y));
            } else if rl.is_mouse_button_released(raylib::consts::MouseButton::MOUSE_LEFT_BUTTON) {
                match arrow.0 {
                    None => (),
                    Some(start) => {
                        let vec = rl.get_mouse_position();
                        let end = Vector2::new(vec.x, vec.y);
                        if (start - end).norm_squared().sqrt() < MIN_THROW {
                            // not far enough
                            arrow.0 = None;
                            return;
                        }
                        if let Some(collider) = physics_world.collider(collider_entity.0) {
                            let mut pos = collider.position().translation;
                            pos.vector.y -= 0.2;
                            // create an arrow

                            let size = 0.05;

                            let vec = (start - end) / zoom_camera.0.zoom * 3.0;
                            let vel = nphysics2d::algebra::Velocity2::new(vec, 0.0);
                            let rb = RigidBodyDesc::new()
                                .translation(pos.vector)
                                .set_velocity(vel)
                                .build();
                            let rb_handle = physics_world.bodies.insert(rb);

                            // Build the collider.
                            let ball_shape = ShapeHandle::new(Ball::new(size));
                            let mut material = nphysics2d::material::BasicMaterial::new(0.1, 0.5);
                            material.restitution_combine_mode =
                                nphysics2d::material::MaterialCombineMode::Multiply;
                            let mh = nphysics2d::material::MaterialHandle::new(material);

                            let co = ColliderDesc::new(ball_shape.clone())
                                .density(1.0)
                                .material(mh)
                                .ccd_enabled(true)
                                .collision_groups(crate::groups::collide_all_but_player())
                                .build(BodyPartHandle(rb_handle, 0));
                            let co_handle = physics_world.colliders.insert(co);

                            let entity = entities.create();
                            sensors
                                .insert(entity, Thrown(arrow.1, crate::groups::PLAYER_GROUP))
                                .unwrap();
                            bodies.insert(entity, Body(rb_handle)).unwrap();
                            colliders.insert(entity, Collider(co_handle)).unwrap();
                            drawables
                                .insert(
                                    entity,
                                    Drawable::Sprite {
                                        name: "ore_coal.png".to_owned(),
                                        scale: 5.0 * size,
                                    },
                                )
                                .unwrap();
                        }
                    }
                }
                arrow.0 = None;
            } else if let Some(initial) = arrow.0 {
                let vec = rl.get_mouse_position();
                let end = Vector2::new(vec.x, vec.y);
                if (initial - end).norm_squared().sqrt() < MIN_THROW {
                    // not far enough
                    skeleton.arm_action = crate::skeletons::component::ArmAction::None;
                    return;
                }
                skeleton.arm_action = crate::skeletons::component::ArmAction::Throw(initial - end);
            } else {
                skeleton.arm_action = crate::skeletons::component::ArmAction::None;
            }
        }
    }
}
