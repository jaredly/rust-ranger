#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, Ground, RigidBodyDesc,
};
// extern crate ketos;
// #[macro_use]
// extern crate ketos_derive;
//
extern crate nalgebra as na;

const BALL_RADIUS: f32 = 0.1;

mod basics;
mod draw;
mod groups;
mod player;
mod scripting;
mod skeletons;
mod sprites;
mod throw;
use basics::*;
use draw::Drawable;
use throw::ArrowSys;

// Can I just define a block as a normal item, but have it have a flag like "static" or something
#[derive(Component)]
struct Block;

fn make_blocks() {}

struct PhysicsMove;

impl<'a> System<'a> for PhysicsMove {
    // this is how we declare our dependencies
    type SystemData = (WriteExpect<'a, PhysicsWorld<f32>>,);

    // this runs the system
    fn run(&mut self, (mut physics_world,): Self::SystemData) {
        physics_world.step();
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
            if physics_world
                .geom
                .colliders_in_contact_with(&physics_world.colliders, collider.0)
                .unwrap()
                .next()
                .is_some()
            {
                remove = true;
            };
            if remove {
                let rb = physics_world.rigid_body_mut(body.0).unwrap();
                rb.enable_gravity(true);
                to_remove.push(entity);
            }
        }
        for entity in to_remove {
            gravities.remove(entity);
        }
    }
}

static BLOCK_SIZE: f32 = 0.4;

fn add_block(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    x: f32,
    y: f32,
) {
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(
        BLOCK_SIZE / 2.0,
        BLOCK_SIZE / 2.0,
    )));
    let ground_collider = ColliderDesc::new(ground_shape)
        .translation(Vector2::new(x, y))
        .collision_groups(groups::member_all_but_player())
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    world
        .create_entity()
        .with(Collider(ground_collider))
        .with(Drawable::Sprite {
            name: "brick_grey.png".into(),
            scale: 0.4,
        })
        .build();
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
        .collision_groups(groups::member_all_but_player())
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
        .with(player::PlayerSys, "player_move", &[])
        .with(
            skeletons::component::SkeletonSys,
            "skeletons",
            &["player_move"],
        )
        .with(PhysicsMove, "p_move", &["player_move"])
        .with(throw::ThrownSys, "sensor_until", &["p_move"])
        .with(ArrowSys, "arrows", &["sensor_until"])
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
            let level = (i / 30) as f32;
            let radius = phys_h / 6.0 * (2.0 + level);
            let i = i % 30;
            let cx = phys_w / 2.0;
            let cy = phys_h / 2.0;
            let theta = i as f32 / 30.0 * std::f32::consts::PI * 2.0;
            let x = theta.cos() * radius + cx;
            let y = theta.sin() * radius + cy;

            // Build the rigid body.
            let mut rb = RigidBodyDesc::new().translation(Vector2::new(x, y)).build();
            use nphysics2d::object::Body;
            rb.enable_gravity(false);
            let rb_handle = physics_world.bodies.insert(rb);

            // Build the collider.
            let co = ColliderDesc::new(ball_shape.clone())
                .density(1.0)
                .collision_groups(groups::member_all_but_player())
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

    player::Player::create_entity(&mut world, &mut physics_world, Vector2::new(3.0, 1.0));

    // Add ground to system
    let ground_handle = physics_world.bodies.insert(Ground::new());

    for i in 0..10 {
        add_block(
            &mut world,
            &mut physics_world,
            ground_handle,
            phys_w / 10.0 * i as f32,
            phys_h - BLOCK_SIZE * 2.0,
        );
    }

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

    let skel_file = "./assets/skeletons.ron";

    let skeletons = skeletons::read(skel_file).unwrap();
    world.add_resource(skeletons);

    let should_close = false;
    let mut skel_change = std::fs::metadata(skel_file).unwrap().modified().unwrap();
    let mut last = std::time::Instant::now();
    while !window_should_close(&world) && !should_close {
        {
            let mut tick = world.write_resource::<basics::Tick>();
            let now = std::time::Instant::now();
            *tick = basics::Tick(now - last);
            last = now;
            let skel_new = std::fs::metadata(skel_file).unwrap().modified().unwrap();
            if skel_new > skel_change {
                let mut skeletons = world.write_resource::<skeletons::Skeletons>();
                match skeletons::read(skel_file) {
                    Ok(skel) => {
                        *skeletons = skel;
                        skel_change = skel_new;
                    }
                    Err(_) => (),
                }
            }
        }
        dispatcher.dispatch(&world.res);
        world.maintain();
    }
}

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<raylib::RaylibHandle>();
    rl.window_should_close()
}
