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
        let mut body = RigidBodyDesc::new().translation(position).build();
        body.set_rotations_kinematic(true);
        let rb = physics_world.bodies.insert(body);
        let collider = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .density(1.0)
            .collision_groups(groups::player())
            .build(BodyPartHandle(rb, 0));
        // this is the only one that's not in the player group
        let sensor = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .sensor(true)
            .collision_groups(groups::member_all_but_player())
            .build(BodyPartHandle(rb, 0));
        let jump_sensor = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .sensor(true)
            .collision_groups(groups::player())
            .translation(Vector2::new(0.0, offset))
            .build(BodyPartHandle(rb, 0));
        let left_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
                .collision_groups(groups::player())
                .translation(Vector2::new(-offset, 0.0))
                .build(BodyPartHandle(rb, 0)),
        );
        let right_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
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
            .collision_groups(groups::player())
            .build(BodyPartHandle(rb, 0)),
        );

        let cb = physics_world.colliders.insert(collider);
        let jcb = physics_world.colliders.insert(jump_sensor);
        let sensor_handle = physics_world.colliders.insert(sensor);
        world
            .create_entity()
            .with(Body(rb))
            .with(throw::ArrowLauncher(None, sensor_handle))
            .with(skeletons::component::Skeleton::new("female"))
            .with(Player {
                down: jcb,
                left: left_sensor,
                right: right_sensor,
                pickup: pickup_sensor,
            })
            .with(Collider(cb))
            // .with(Drawable::Sprite {
            //     name: "gnome_head.png".to_owned(),
            //     scale: 0.4,
            // })
            .build();
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
