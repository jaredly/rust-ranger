#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate lazy_static;

// For scripting
extern crate pest;
#[macro_use]
extern crate pest_derive;


use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use nphysics2d::object::{BodyPartHandle, ColliderDesc, DefaultBodyHandle, Ground, RigidBodyDesc};
extern crate nalgebra as na;

pub static WORLD_WIDTH: f32 = 100.0;
pub const BALL_RADIUS: f32 = 0.1;

// mod new_scripting;
mod basics;
mod config;
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
// #[derive(Component)]
// struct Block;

fn make_blocks(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    phys_w: f32,
    phys_h: f32,
) {
    let w = WORLD_WIDTH / BLOCK_SIZE;

    for y in 1..8 {
        for i in 0..w as usize {
            add_block(
                world,
                physics_world,
                ground_handle,
                BLOCK_SIZE * i as f32, // * 1.01,
                phys_h - BLOCK_SIZE * 3.0 + BLOCK_SIZE * y as f32,
            );
        }
    }
}

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
                .rigid_body(body.0)
                .unwrap()
                .part(0)
                .unwrap()
                .velocity()
                .linear
                .norm_squared()
                > 0.0
            {
                remove = true;
            }
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
    let builder = world.create_entity();
    let ground_collider = ColliderDesc::new(ground_shape)
        .user_data(builder.entity)
        .translation(Vector2::new(x, y))
        .collision_groups(groups::member_all_but_player())
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    builder
        .with(Collider(ground_collider))
        .with(Drawable::Sprite {
            name: "brick_grey.png".into(),
            scale: 0.4,
        })
        .build();
}

fn add_wall(
    world: &mut World,
    physics_world: &mut PhysicsWorld<f32>,
    ground_handle: DefaultBodyHandle,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let builder = world.create_entity();
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(w / 2.0, h / 2.0)));
    let ground_collider = ColliderDesc::new(ground_shape)
        .user_data(builder.entity)
        .translation(Vector2::new(x, y))
        .collision_groups(groups::member_all_but_player())
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    builder
        .with(Collider(ground_collider))
        .with(Drawable::Rect {
            color: raylib::color::Color::BLACK,
            width: w,
            height: h,
        })
        // .with(Drawable::Sprite {
        //     name: "brick_grey.png".into(),
        //     scale: 0.4,
        // })
        .build();
}

pub struct ZoomCamera(pub raylib::camera::Camera2D);
impl Default for ZoomCamera {
    fn default() -> ZoomCamera {
        ZoomCamera(raylib::camera::Camera2D {
            target: raylib::math::Vector2::new(0.0, 0.0),
            offset: raylib::math::Vector2::new(0.0, 0.0),
            rotation: 0.0,
            zoom: config::with(|config| config.zoom),
        })
    }
}

pub struct Camera {
    pos: Vector2<f32>,
}
impl Default for Camera {
    fn default() -> Camera {
        Camera {
            pos: Vector2::new(0.0, 0.0),
        }
    }
}

struct CameraFollowSys;
impl<'a> System<'a> for CameraFollowSys {
    type SystemData = (
        Write<'a, Camera>,
        Read<'a, config::Config>,
        Read<'a, ZoomCamera>,
        ReadExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, player::Player>,
        ReadStorage<'a, Collider>,
    );

    fn run(
        &mut self,
        (mut camera, config, zoom_camera, physics, players, colliders): Self::SystemData,
    ) {
        let view_width = config.screen_size as f32 / zoom_camera.0.zoom;
        let offset = config.screen_size as f32 / zoom_camera.0.zoom / 2.0;
        let margin = offset / 6.0;
        for (_player, collider) in (&players, &colliders).join() {
            let collider = physics.collider(collider.0).unwrap();
            let p = collider.position().translation;
            camera.pos.x = camera
                .pos
                .x
                .max(p.x - offset + margin - offset)
                .min(p.x - offset - margin + offset);
            camera.pos.y = camera
                .pos
                .y
                .max(p.y - offset + margin - offset)
                .min(p.y - offset - margin + offset);
            // camera.pos = Vector2::new(p.x - offset, p.y - offset);
        }
    }
}

fn main() {
    // screen
    let screen_w = config::with(|c| c.screen_size);
    let screen_h = config::with(|c| c.screen_size);

    let (mut rl, thread) = raylib::init()
        .size(screen_w as i32, screen_h as i32)
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
    sprites.add(&mut rl, &thread, "assets/extras.png", "assets/extras.xml");

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
        .with(CameraFollowSys, "camera_follow", &["p_move"])
        .with(player::PickupSys, "pickup", &["p_move"])
        .with(ArrowSys, "arrows", &["sensor_until"])
        .with(GravitySys, "gravity_on_collide", &["p_move"])
        .with(throw::FletchingSys, "fletching", &["p_move"])
        .with_thread_local(draw::Draw { thread })
        .build();

    dispatcher.setup(&mut world.res);

    let mut physics_world: PhysicsWorld<f32> = PhysicsWorld::new();

    let ball_shape = ShapeHandle::new(Ball::new(BALL_RADIUS));

    let zoom = ZoomCamera::default().0.zoom;
    let phys_w = screen_w as f32 / zoom;
    let phys_h = screen_h as f32 / zoom;

    // three rings of apples
    for i in 0..90 {
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

        let builder = world.create_entity();

        // Build the collider.
        let co = ColliderDesc::new(ball_shape.clone())
            .density(1.0)
            .user_data(builder.entity)
            .collision_groups(groups::member_all_but_player())
            .build(BodyPartHandle(rb_handle, 0));
        let co_handle = physics_world.colliders.insert(co);

        builder
            .with(Body(rb_handle))
            .with(Collider(co_handle))
            .with(GravityOnCollide)
            .with(Drawable::Sprite {
                name: "apple.png".to_owned(),
                scale: 0.4,
            })
            .build();
    }

    player::Player::create_entity(
        &mut world,
        &mut physics_world,
        Vector2::new(3.0, phys_h * 0.8),
    );

    // Add ground to system
    let ground_handle = physics_world.bodies.insert(Ground::new());

    add_wall(
        &mut world,
        &mut physics_world,
        ground_handle,
        -0.3,
        -100.0,
        0.1,
        300.0,
    );
    add_wall(
        &mut world,
        &mut physics_world,
        ground_handle,
        WORLD_WIDTH,
        -100.0,
        0.1,
        300.0,
    );

    make_blocks(
        &mut world,
        &mut physics_world,
        ground_handle,
        phys_w,
        phys_h,
    );

    world.add_resource(physics_world);
    world.add_resource(sprites);
    world.add_resource(rl);

    world.add_resource(config::Config::default());
    let mut config_change = std::fs::metadata(config::CONFIG_FILE)
        .unwrap()
        .modified()
        .unwrap();

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
            {
                let config_new = std::fs::metadata(config::CONFIG_FILE)
                    .unwrap()
                    .modified()
                    .unwrap();
                if config_new > config_change {
                    let mut config = world.write_resource::<config::Config>();
                    match config::read(config::CONFIG_FILE) {
                        Ok(conf) => {
                            println!("Reload config");
                            *config = conf.clone();
                            config::with(|config| *config = conf);
                            config_change = config_new;
                        }
                        Err(_) => (),
                    }
                }
            }

            {
                let skel_new = std::fs::metadata(skel_file).unwrap().modified().unwrap();
                if skel_new > skel_change {
                    let mut skeletons = world.write_resource::<skeletons::Skeletons>();
                    match skeletons::read(skel_file) {
                        Ok(skel) => {
                            println!("Reload skeletons");
                            *skeletons = skel;
                            skel_change = skel_new;
                        }
                        Err(_) => (),
                    }
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
