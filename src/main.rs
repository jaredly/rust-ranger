#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, Ground, RigidBodyDesc,
};

extern crate nalgebra as na;

const BOX_SIZE_WIDTH: f32 = 7.0;
const BOX_SIZE_HEIGHT: f32 = 0.2;
const BALL_RADIUS: f32 = 0.1;

mod basics;
mod draw;
mod sprites;
use basics::*;
use draw::Drawable;
mod throw;
use throw::ArrowSys;

struct PhysicsMove;

impl<'a> System<'a> for PhysicsMove {
    // this is how we declare our dependencies
    type SystemData = (WriteExpect<'a, PhysicsWorld<f32>>,);

    // this runs the system
    fn run(&mut self, (mut physics_world,): Self::SystemData) {
        physics_world.step();
    }
}

#[derive(Component)]
struct Player {
    down: DefaultColliderHandle,
    left: DefaultColliderHandle,
    right: DefaultColliderHandle,
}

impl Player {
    fn create_entity(
        world: &mut World,
        physics_world: &mut PhysicsWorld<f32>,
        position: Vector2<f32>,
    ) {
        let height = 0.1;
        let width = 0.1;
        let offset = 0.05;
        let mut body = RigidBodyDesc::new().translation(position).build();
        body.set_rotations_kinematic(true);
        let rb = physics_world.bodies.insert(body);
        let collider = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .density(1.0)
            .build(BodyPartHandle(rb, 0));
        let jump_sensor = ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
            .sensor(true)
            .translation(Vector2::new(0.0, offset))
            .build(BodyPartHandle(rb, 0));
        let left_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
                .translation(Vector2::new(-offset, 0.0))
                .build(BodyPartHandle(rb, 0)),
        );
        let right_sensor = physics_world.colliders.insert(
            ColliderDesc::new(ShapeHandle::new(Capsule::new(height, width)))
                .sensor(true)
                .translation(Vector2::new(offset, 0.0))
                .build(BodyPartHandle(rb, 0)),
        );
        let cb = physics_world.colliders.insert(collider);
        let jcb = physics_world.colliders.insert(jump_sensor);
        world
            .create_entity()
            .with(Body(rb))
            .with(throw::ArrowLauncher(None))
            .with(Player {
                down: jcb,
                left: left_sensor,
                right: right_sensor,
            })
            .with(Collider(cb))
            .with(Drawable::Sprite {
                name: "gnome_head.png".to_owned(),
                scale: 0.4,
            })
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
        false
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
struct GravityOnCollide;

struct GravitySys;

impl<'a> System<'a> for GravitySys {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Body>,
        WriteStorage<'a, GravityOnCollide>,
    );

    fn run(
        &mut self,
        (entities, mut physics_world, colliders, bodies, mut gravities): Self::SystemData,
    ) {
        let mut to_remove = vec![];
        for (entity, _, body, collider) in (&entities, &gravities, &bodies, &colliders).join() {
            let mut remove = false;
            if let Some(_) = physics_world
                .geom
                .colliders_in_contact_with(&physics_world.colliders, collider.0)
                .unwrap()
                .next()
            {
                remove = true;
            }
            if remove {
                let rb = physics_world.rigid_body_mut(body.0).unwrap();
                rb.enable_gravity(true);
                to_remove.push(entity.clone());
            }
        }
        for entity in to_remove {
            gravities.remove(entity);
        }
    }
}

struct PlayerSys;

impl<'a> System<'a> for PlayerSys {
    type SystemData = (
        // Entities<'a>,
        ReadExpect<'a, raylib::RaylibHandle>,
        WriteExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, (rl, mut physics, body, player): Self::SystemData) {
        use raylib::consts::KeyboardKey::*;

        let speed = 0.5;
        let jump_speed = speed * 10.0;
        let max_speed = 3.0;

        for (body, player) in (&body, &player).join() {
            let v = {
                let body = physics.rigid_body_mut(body.0).unwrap();
                let part = body.part(0).unwrap();
                part.velocity().linear
            };

            let mut push = Vector2::new(0.0, 0.0);
            if rl.is_key_down(KEY_W) && player.can_jump(&physics, &body.0) && v.y > -jump_speed {
                let max_jump = -jump_speed - v.y;
                push.y += max_jump;
            }
            if rl.is_key_down(KEY_D) && player.can_go_right(&physics, &body.0) {
                push.x += speed;
            }
            if rl.is_key_down(KEY_A) && player.can_go_left(&physics, &body.0) {
                push.x -= speed;
            }
            if rl.is_key_down(KEY_S) {
                push.y += speed;
            }
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

fn add_ground(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(w / 2.0, h / 2.0)));
    let ground_collider = ColliderDesc::new(ground_shape)
        .translation(Vector2::new(x, y))
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    world
        .create_entity()
        .with(Collider(ground_collider))
        .with(Drawable::Rect {
            color: raylib::color::Color::BLACK,
            width: w,
            height: h,
        })
        .build();
}

fn main() {
    // screen
    let screen_w = 640;
    let screen_h = 480;

    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Examples")
        .build();
    rl.set_target_fps(60);

    let mut sprites = sprites::SpriteSheet::new();
    sprites.add(
        &mut rl,
        &thread,
        "assets/spritesheet_items.png",
        "assets/spritesheet_items.xml",
    );
    sprites.add(&mut rl, &thread, "assets/extras.png", "assets/extras.xml");
    sprites.add(
        &mut rl,
        &thread,
        "assets/spritesheet_characters.png",
        "assets/spritesheet_characters.xml",
    );
    sprites.add(
        &mut rl,
        &thread,
        "assets/spritesheet_particles.png",
        "assets/spritesheet_particles.xml",
    );
    sprites.add(
        &mut rl,
        &thread,
        "assets/spritesheet_tiles.png",
        "assets/spritesheet_tiles.xml",
    );

    let mut world = World::new();

    let mut dispatcher = DispatcherBuilder::new()
        .with(PlayerSys, "player_move", &[])
        .with(ArrowSys, "arrows", &[])
        .with(PhysicsMove, "p_move", &["player_move"])
        .with(GravitySys, "gravity_on_collide", &["p_move"])
        .with_thread_local(draw::Draw { thread })
        .build();

    dispatcher.setup(&mut world.res);

    let mut physics_world: PhysicsWorld<f32> = PhysicsWorld::new();

    let ball_shape = ShapeHandle::new(Ball::new(BALL_RADIUS));

    let phys_w = screen_w as f32 / draw::WORLD_SCALE;
    let phys_h = screen_h as f32 / draw::WORLD_SCALE;

    // three rings of apples
    let ball_handles: Vec<_> = (0..90)
        .map(|i| {
            let l = (i / 30) as f32;
            let r = phys_h / 6.0 * (2.0 + l);
            let i = i % 30;
            let cx = phys_w / 2.0;
            let cy = phys_h / 2.0;
            let t = i as f32 / 30.0 * std::f32::consts::PI * 2.0;
            let x = t.cos() * r + cx;
            let y = t.sin() * r + cy;

            // Build the rigid body.
            let mut rb = RigidBodyDesc::new().translation(Vector2::new(x, y)).build();
            use nphysics2d::object::Body;
            rb.enable_gravity(false);
            let rb_handle = physics_world.bodies.insert(rb);

            // Build the collider.
            let co = ColliderDesc::new(ball_shape.clone())
                .density(1.0)
                .build(BodyPartHandle(rb_handle, 0));
            let co_handle = physics_world.colliders.insert(co);

            (rb_handle, co_handle)
        })
        .collect();
    for (body, collider) in ball_handles {
        world
            .create_entity()
            .with(Body(body))
            .with(Collider(collider))
            .with(GravityOnCollide)
            .with(Drawable::Sprite {
                name: "apple.png".to_owned(),
                scale: 0.4,
            })
            .build();
    }

    Player::create_entity(&mut world, &mut physics_world, Vector2::new(3.0, 1.0));

    // Add ground to system
    let ground_handle = physics_world.bodies.insert(Ground::new());

    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w / 6.0 * 5.0,
        phys_h / 4.0 * 3.0,
        phys_w / 3.0,
        0.10,
    );

    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w / 2.0,
        phys_h / 2.0,
        phys_w / 3.0,
        0.10,
    );

    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w / 6.0,
        phys_h / 4.0,
        phys_w / 3.0,
        0.10,
    );

    // bottom
    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w / 2.0,
        phys_h,
        phys_w,
        0.10,
    );
    // top
    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w / 2.0,
        0.0,
        phys_w,
        0.10,
    );
    // l
    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        0.0,
        phys_h / 2.0,
        0.1,
        phys_h,
    );
    // r
    add_ground(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w,
        phys_h / 2.0,
        0.1,
        phys_h,
    );

    world.add_resource(physics_world);
    world.add_resource(sprites);
    world.add_resource(rl);

    let should_close = false;
    while !window_should_close(&world) && !should_close {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
}

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<raylib::RaylibHandle>();
    rl.window_should_close()
}
