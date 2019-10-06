use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, RigidBodyDesc,
};

use crate::basics::*;

use crate::groups;
use crate::skeletons;
use crate::throw;

#[derive(Component)]
pub struct Player {
    down: DefaultColliderHandle,
    left: DefaultColliderHandle,
    right: DefaultColliderHandle,
    pub pickup: DefaultColliderHandle,
    pickup_cooldown: f32,
}

impl Player {
    pub fn create_entity(
        world: &mut World,
        physics_world: &mut PhysicsWorld<f32>,
        position: Vector2<f32>,
    ) {
        let height = 0.3;
        let width = 0.1;
        let offset = 0.07;

        let builder = world.create_entity();

        let mut body = RigidBodyDesc::new().translation(position).build();
        body.set_rotations_kinematic(true);
        let rb = physics_world.bodies.insert(body);
        let collider = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .density(1.0)
            .user_data(builder.entity)
            .collision_groups(groups::player())
            .build(BodyPartHandle(rb, 0));
        // this is the only one that's not in the player group
        let sensor = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .sensor(true)
            .user_data(builder.entity)
            .collision_groups(groups::member_all_but_player())
            .build(BodyPartHandle(rb, 0));
        let jump_sensor = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .sensor(true)
            .user_data(builder.entity)
            .collision_groups(groups::player())
            .translation(Vector2::new(0.0, offset))
            .build(BodyPartHandle(rb, 0));
        let left_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
                .user_data(builder.entity)
                .collision_groups(groups::player())
                .translation(Vector2::new(-offset, 0.0))
                .build(BodyPartHandle(rb, 0)),
        );
        let right_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
                .user_data(builder.entity)
                .collision_groups(groups::player())
                .translation(Vector2::new(offset, 0.0))
                .build(BodyPartHandle(rb, 0)),
        );

        let pickup_margin = 0.1;
        let pickup_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(
                height - pickup_margin,
                width + pickup_margin * 4.0,
            )))
            .sensor(true)
            .user_data(builder.entity)
            .collision_groups(groups::player())
            .build(BodyPartHandle(rb, 0)),
        );

        let cb = physics_world.colliders.insert(collider);
        let jcb = physics_world.colliders.insert(jump_sensor);
        let sensor_handle = physics_world.colliders.insert(sensor);
        builder
            .with(Body(rb))
            .with(throw::ArrowLauncher(None, sensor_handle))
            .with(skeletons::component::Skeleton::new("female"))
            .with(Player {
                down: jcb,
                left: left_sensor,
                right: right_sensor,
                pickup: pickup_sensor,
                pickup_cooldown: 0.0,
            })
            .with(Collider(cb))
            // .with(Drawable::Sprite {
            //     name: "gnome_head.png".to_owned(),
            //     scale: 0.4,
            // })
            .build();
    }

    pub fn closest_pickupable_entity(
        &self,
        physics: &PhysicsWorld<f32>,
        player_collider: DefaultColliderHandle,
    ) -> Option<(DefaultColliderHandle, Entity, na::Vector2<f32>)> {
        let player_pos = physics.collider(player_collider).unwrap().position();
        let mut closest = None;
        for (handle, collider) in physics
            .geom
            .colliders_in_proximity_of(&physics.colliders, self.pickup)
            .unwrap()
        {
            if collider.is_sensor() || handle == player_collider {
                continue;
            }
            let body = physics.rigid_body(collider.body()).unwrap();
            if body.is_ground() {
                continue;
            }
            if let Some(data) = collider.user_data() {
                if let Some(entity) = data.downcast_ref::<Entity>() {
                    let to_vec =
                        player_pos.translation.vector - collider.position().translation.vector;
                    let dist = (to_vec).norm_squared().sqrt();
                    match closest {
                        Some((_, d)) if d < dist => (),
                        _ => closest = Some(((handle, *entity, to_vec), dist)),
                    }
                } else {
                    // println!("Not an entity {:?}", data.type_id())
                }
            } else {
                // println!("No data")
            }
        }
        match closest {
            None => None,
            Some((res, _)) => Some(res),
        }
    }

    fn can_go_left(&self, physics: &PhysicsWorld<f32>, body: &DefaultBodyHandle) -> bool {
        for (_handle, collider) in physics
            .geom
            .colliders_in_proximity_of(&physics.colliders, self.left)
            .unwrap()
        {
            let bh = collider.body();
            if &bh == body {
                continue;
            }
            let body = physics.rigid_body(bh).unwrap();
            if let Some(part) = body.part(0) {
                if part.is_ground() {
                    return false;
                }
            }
        }
        true
    }

    fn can_go_right(&self, physics: &PhysicsWorld<f32>, body: &DefaultBodyHandle) -> bool {
        for (_handle, collider) in physics
            .geom
            .colliders_in_proximity_of(&physics.colliders, self.right)
            .unwrap()
        {
            let bh = collider.body();
            if &bh == body {
                continue;
            }
            let body = physics.rigid_body(bh).unwrap();
            // TODO if there are multiple body parts?
            if let Some(part) = body.part(0) {
                if part.is_ground() {
                    return false;
                }
            }
        }
        true
    }

    fn can_jump(&self, physics: &PhysicsWorld<f32>, body: &DefaultBodyHandle) -> bool {
        for (_handle, collider) in physics
            .geom
            .colliders_in_proximity_of(&physics.colliders, self.down)
            .unwrap()
        {
            let bh = collider.body();
            if &bh == body {
                continue;
            }
            let body = physics.rigid_body(bh).unwrap();
            if let Some(part) = body.part(0) {
                if part.is_ground() || part.velocity().linear.y.abs() < 0.1 {
                    return true;
                }
            }
        }
        if !self.can_go_left(physics, body) && !self.can_go_right(physics, body) {
            return true;
        }
        false
    }
}

pub struct PickupSys;
impl<'a> System<'a> for PickupSys {
    type SystemData = (
        Read<'a, Tick>,
        Entities<'a>,
        ReadExpect<'a, raylib::RaylibHandle>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, skeletons::component::Skeleton>,
        // stuff to remove
        WriteStorage<'a, Body>,
        WriteStorage<'a, throw::Thrown>,
        WriteStorage<'a, Collider>,
        WriteStorage<'a, crate::draw::Drawable>,
    );

    fn run(
        &mut self,
        (
            tick,
            entities,
            rl,
            mut physics_world,
            mut players,
            mut skeletons,
            mut bodies,
            mut throwns,
            mut colliders,
            mut drawables,
        ): Self::SystemData,
    ) {
        use raylib::consts::KeyboardKey::*;
        let mut to_remove = None;
        fn empty_vec(left: bool) -> Vector2<f32> {
            let a = crate::config::with(|config| config.pickup_empty_angle);
            let a =
                a / 180.0 * std::f32::consts::PI + if left { std::f32::consts::PI } else { 0.0 };
            Vector2::new(a.cos(), a.sin())
        }
        if let Some((player, skeleton, player_collider)) =
            (&mut players, &mut skeletons, &colliders).join().next()
        {
            if rl.is_key_down(KEY_C) {
                if player.pickup_cooldown > 0.0 {
                    let tick = tick.0.as_micros() as f32 / 1000.0;
                    player.pickup_cooldown -= tick;

                    // point to the next thing
                    if player.pickup_cooldown < crate::config::with(|config| config.pickup_switch) {
                        if let Some((collider_handle, entity, to_vec)) =
                            player.closest_pickupable_entity(&physics_world, player_collider.0)
                        {
                            // skeleton.pointing = Some(to_vec);
                            to_remove = Some((collider_handle, entity));
                            skeleton.pointing = Some(to_vec);
                            player.pickup_cooldown =
                                crate::config::with(|config| config.pickup_cooldown);
                        //
                        } else {
                            skeleton.pointing = Some(empty_vec(
                                skeleton.facing == skeletons::component::Facing::Left,
                            ))
                        }
                    }
                } else if let Some((collider_handle, entity, to_vec)) =
                    player.closest_pickupable_entity(&physics_world, player_collider.0)
                {
                    to_remove = Some((collider_handle, entity));
                    skeleton.pointing = Some(to_vec);
                    player.pickup_cooldown = crate::config::with(|config| config.pickup_cooldown);
                //
                } else {
                    skeleton.pointing = Some(empty_vec(
                        skeleton.facing == skeletons::component::Facing::Left,
                    ))
                }
            } else {
                player.pickup_cooldown = 0.0;
                skeleton.pointing = None;
            }
        }
        if let Some((collider_handle, entity)) = to_remove {
            let Body(body_handle) = bodies.get(entity).unwrap();
            physics_world.bodies.remove(*body_handle);
            bodies.remove(entity);

            physics_world.colliders.remove(collider_handle);
            colliders.remove(entity);

            drawables.remove(entity);
            throwns.remove(entity);

            entities.delete(entity);
        }
    }
}

pub struct PlayerSys;

impl<'a> System<'a> for PlayerSys {
    type SystemData = (
        // Entities<'a>,
        ReadExpect<'a, raylib::RaylibHandle>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        WriteStorage<'a, skeletons::component::Skeleton>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, (rl, mut physics, mut skeletons, body, player): Self::SystemData) {
        use raylib::consts::KeyboardKey::*;

        let speed = 0.5;
        let jump_speed = speed * 10.0;
        let max_speed = 3.0;

        for (body, player, skeleton) in (&body, &player, &mut skeletons).join() {
            let v = {
                let body = physics.rigid_body_mut(body.0).unwrap();
                let part = body.part(0).unwrap();
                part.velocity().linear
            };

            let mut push = Vector2::new(0.0, 0.0);
            fn on_ground(player: &Player, physics: &PhysicsWorld<f32>, body: &Body) -> bool {
                player.can_jump(&physics, &body.0)
            };
            if skeleton.action == skeletons::component::Action::Jump
                && v.y > 0.0
                && on_ground(&player, &physics, &body)
            {
                println!("Reset to standing");
                skeleton.set_action(skeletons::component::Action::Stand);
            }
            if rl.is_key_down(KEY_D) && player.can_go_right(&physics, &body.0) {
                skeleton.face(skeletons::component::Facing::Right);
                push.x += speed;
                if skeleton.is_standing() && on_ground(&player, &physics, &body) {
                    println!("Walk");
                    skeleton.set_action(skeletons::component::Action::Walk);
                }
            }
            if rl.is_key_down(KEY_A) && player.can_go_left(&physics, &body.0) {
                skeleton.face(skeletons::component::Facing::Left);
                if skeleton.is_standing() && on_ground(&player, &physics, &body) {
                    println!("Walk");
                    skeleton.set_action(skeletons::component::Action::Walk);
                }
                push.x -= speed;
            }
            if rl.is_key_down(KEY_W) && on_ground(&player, &physics, &body) && v.y > -jump_speed {
                let max_jump = -jump_speed - v.y;
                println!("Set to jumping");
                skeleton.set_action(skeletons::component::Action::Jump);
                push.y += max_jump;
            }
            use skeletons::component::{ArmAction, SwingDirection};
            if rl.is_key_down(KEY_SPACE) {
                let (position, forward, object) = if let ArmAction::Swing { position, forward, object, ..} = &skeleton.arm_action {
                    (*position, *forward, object.clone())
                } else {
                    (0.0, true, "pick_bronze.png".to_owned())
                };
                let (position, forward) = advance_swing(position, forward);
                skeleton.arm_action = ArmAction::Swing {position, forward, object,
                direction: if rl.is_key_down(KEY_W) {
                    SwingDirection::Up
                } else if rl.is_key_down(KEY_S) {
                    SwingDirection::Down
                } else {
                    SwingDirection::Forward
                }
                };
            } else if let ArmAction::Swing {..} = &skeleton.arm_action {
                skeleton.arm_action = ArmAction::None;
            }
            // if rl.is_key_down(KEY_S) {
            //     push.y += speed;
            // }
            if push.x == 0.0 && push.y == 0.0 {
                continue;
            }

            if push.x > 0.0 && v.x > max_speed {
                push.x = 0.0;
            }
            if push.x < 0.0 && v.x < -max_speed {
                push.x = 0.0;
            }
            let body = physics.rigid_body_mut(body.0).unwrap();
            body.apply_force(
                0,
                &nphysics2d::algebra::Force2::linear(push),
                nphysics2d::algebra::ForceType::VelocityChange,
                true,
            );
        }
    }
}

fn advance_swing(mut position: f32, forward: bool) -> (f32, bool) {
    if forward {
        position += 0.07;
        if position > 1.0 {
            (1.0, false)
        } else {
            (position, forward)
        }
    } else {
        position -= 0.05;
        if position <= 0.0 {
            (0.0, true)
        } else {
            (position, forward)
        }
    }
}
