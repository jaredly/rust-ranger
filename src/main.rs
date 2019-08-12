#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

use nalgebra::{Point2, Vector2};
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use nphysics2d::object::{DefaultBodyHandle, ColliderDesc, DefaultColliderHandle, RigidBodyDesc, Ground, BodyPartHandle};

extern crate nalgebra as na;

use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};


use std::sync::Arc;

const BOX_SIZE_WIDTH: f32 = 3.0;
const BOX_SIZE_HEIGHT: f32 = 0.2;
const BALL_RADIUS: f32 = 0.1;

mod sprites;

#[derive(Component)]
enum Drawable {
    Sprite {
        name: String,
        scale: f32,
    },
    Rect {
        color: raylib::Color,
        width: f32,
        height: f32,
    }
}

#[derive(Component)]
struct Collider(DefaultColliderHandle);

#[derive(Component)]
struct Body(DefaultBodyHandle);

struct PhysicsMove;

struct PhysicsWorld<N: na::RealField> {
    mech: DefaultMechanicalWorld<N>,
    geom: DefaultGeometricalWorld<N>,
    bodies: DefaultBodySet<N>,
    colliders: DefaultColliderSet<N>,
    joints: DefaultJointConstraintSet<N>,
    forces: DefaultForceGeneratorSet<N>,
}

impl PhysicsWorld<f32> {
    fn new() -> Self {
        let mech = DefaultMechanicalWorld::new(Vector2::new(0.0, 9.81));
        let geom = DefaultGeometricalWorld::new();

        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let joints = DefaultJointConstraintSet::new();
        let forces = DefaultForceGeneratorSet::new();
        PhysicsWorld {mech, geom, bodies, colliders, joints, forces}
    }
}

impl<N: na::RealField> PhysicsWorld<N> {
    fn step(&mut self) {
        self.mech.step(
                &mut self.geom,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.joints,
                &mut self.forces,
            );
    }

    fn collider(&self, handle: DefaultColliderHandle) -> Option<&nphysics2d::object::Collider<N, DefaultColliderHandle>> {
        self.colliders.get(handle)
    }

    fn rigid_body(&self, handle: DefaultBodyHandle) -> Option<&dyn nphysics2d::object::Body<N>> {
        self.bodies.get(handle)
    }

    fn rigid_body_mut(&mut self, handle: DefaultBodyHandle) -> Option<&mut dyn nphysics2d::object::Body<N>> {
        self.bodies.get_mut(handle)
    }
}

impl<'a> System<'a> for PhysicsMove {
    // this is how we declare our dependencies
    type SystemData = (WriteExpect<'a, PhysicsWorld<f32>>,);

    // this runs the system
    fn run(&mut self, (mut physics_world,): Self::SystemData) {
        physics_world.step();
    }
}

struct Draw;

impl<'a> System<'a> for Draw {
    type SystemData = (
        WriteExpect<'a, Arc<raylib::RaylibHandle>>,
        ReadExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Drawable>,
        ReadExpect<'a, sprites::SpriteSheet>,
    );

    fn run(&mut self, (rl, physics, colliders, bodies, drawables, sheet): Self::SystemData) {
        use specs::Join;

        let world_scale = 100.0;

        for (collider, drawable) in (&colliders, &drawables).join() {
            if let Some(collider) = physics.collider(collider.0) {
                let p = collider.position() * Point2::new(0.0, 0.0);
                let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;

                match drawable {
                    Drawable::Sprite {name, scale} => {
                        sheet.draw(rl.as_ref(), &name, (p.x * world_scale, p.y * world_scale), r, scale * world_scale);
                    },
                    Drawable::Rect { color, width, height } => {
                        rl.draw_rectangle_v(
                            ((p.x - width / 2.0)  * world_scale, (p.y - height / 2.0) * world_scale),
                            (width  * world_scale, height * world_scale),
                            *color,
                        );

                    }
                }
            }
        }

        rl.draw_fps(5, 5);
    }
}

#[derive(Component)]
struct Player;

// struct PlayerSys;

// impl<'a> System<'a> for PlayerSys {
//     type SystemData = (
//         // Entities<'a>,
//         ReadExpect<'a, raylib::RaylibHandle>,
//         WriteExpect<'a, PhysicsWorld<f32>>,
//         Read<'a, Body>,
//         Read<'a, Player>,
//     );

//     fn run(&mut self, (rl, physics, body, player): Self::SystemData) {
//         use raylib::consts::KeyboardKey::*;
//         use specs::Join;

//         let body = physics.rigid_body_mut(body.0).unwrap();
//         body.apply_local_force(0, &nphysics2d::algebra::Force2::linear(push), nphysics2d::algebra::ForceType::Impulse, true);
//         // let vel = body.velocity();
//         // let new_vel = vel.linear + push;
//         // body.set_linear_velocity(new_vel);
//         // let player = (&*ents, &pos, &players).join().nth(0).unwrap();

//         // let mut new_pos = player.1.clone();
//         // if rl.is_key_pressed(KEY_D) {
//         //     new_pos.0 += 1;
//         // } else if rl.is_key_pressed(KEY_A) {
//         //     new_pos.0 -= 1;
//         // } else if rl.is_key_pressed(KEY_W) {
//         //     new_pos.1 -= 1;
//         // } else if rl.is_key_pressed(KEY_S) {
//         //     new_pos.1 += 1;
//         // } else {
//         //     return;
//         // }

//         // let p_ent = player.0;

//         // match emap.get(&new_pos.into()) {
//         //     Some(e) => {
//         //         players.insert(*e, Player).unwrap();
//         //         players.remove(p_ent);
//         //     }
//         //     _ => println!("Nothing"),
//         // }
//     }
// }



fn main() {
    let mut world = World::new();
    let mut physics_world: PhysicsWorld<f32> = PhysicsWorld::new();

    world.register::<Collider>();
    world.register::<Drawable>();
    world.register::<Body>();

    // Set up physics world (the hard part) first
    // Raylib uses top left as 0,0 so down is really increasing
    // physics_world.set_gravity(Vector2::new(0.0, 9.81));

    // Create the ground
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(BOX_SIZE_WIDTH / 2.0, BOX_SIZE_HEIGHT / 2.0)));

    // Add ground to system
    let ground_handle = physics_world.bodies.insert(Ground::new());
    let ground_collider = ColliderDesc::new(ground_shape)
        .translation(Vector2::new(3.20, 3.40))
        .build(BodyPartHandle(ground_handle, 0));
    physics_world.colliders.insert(ground_collider);
    // ground

    // Create a couple of balls with actual rigidbodies (only rigidbodies have gravity)

    let ball_shape = ShapeHandle::new(Ball::new(BALL_RADIUS));
    // let collider = ColliderDesc::new(ball_shape).density(1.0);
    // let mut rigid_body = RigidBodyDesc::new().collider(&collider);

    // Note that for rust the last statement without a semicolon is the return
    let ball_handles: Vec<_> = (0..15)
        .map(|i| {

            let x = 3.0 + i as f32 * ((BALL_RADIUS + ColliderDesc::<f32>::default_margin()) * 2.0 - 0.05);
            let y = 1.0 + i as f32 * 0.1;

            // Build the rigid body.
            let rb = RigidBodyDesc::new()
                .translation(Vector2::new(x, y))
                .build();
            let rb_handle = physics_world.bodies.insert(rb);

            // Build the collider.
            let co = ColliderDesc::new(ball_shape.clone())
                .density(1.0)
                .build(BodyPartHandle(rb_handle, 0));
            let co_handle = physics_world.colliders.insert(co);

            (rb_handle, co_handle)
            // We use the rigid_body as a template to insert balls into the world
            // rigid_body
            //     .set_translation(Vector2::new(x, y))
            //     .build(&mut physics_world)
            //     .handle()
        })
        .collect();
    for (body, collider) in ball_handles {
        world.create_entity()
            .with(Body(body))
            .with(Collider(collider))
            .with(Drawable::Sprite {name: "apple.png".to_owned(), scale: 0.4})
            .build();
    }

    world.add_resource(physics_world);

    // Hard part over, populate the specs world
    // First we register our components and physics_world
    world.create_entity()
        .with(Collider(ground_handle))
        .with(Drawable::Rect { color: raylib::Color::BLACK, width: BOX_SIZE_WIDTH, height: BOX_SIZE_HEIGHT})
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(PhysicsMove, "p_move", &[])
        // make the physics system a dependency of drawing
        .with(Draw, "draw", &["p_move"])
        .build();

    // Now we setup raylib for the drawing

    let w = 640;
    let h = 480;
    // Use an reference counted pointer to share raylib between this thread and the drawing one
    let rl = Arc::new(raylib::init().size(w, h).title("Examples").build());
    rl.set_target_fps(60);

    let mut sprites = sprites::SpriteSheet::new();
    sprites.add(&rl, "assets/spritesheet_items.png", "assets/spritesheet_items.xml");
    sprites.add(&rl, "assets/extras.png", "assets/extras.xml");
    sprites.add(&rl, "assets/spritesheet_characters.png", "assets/spritesheet_characters.xml");
    sprites.add(&rl, "assets/spritesheet_particles.png", "assets/spritesheet_particles.xml");
    sprites.add(&rl, "assets/spritesheet_tiles.png", "assets/spritesheet_tiles.xml");

    world.add_resource(sprites);

    world.add_resource(rl.clone());

    dispatcher.setup(&mut world.res);

    let mut should_close = false;
    while !rl.window_should_close() && !should_close {
        rl.begin_drawing();
        rl.clear_background(raylib::Color::WHITE);
        dispatcher.dispatch(&mut world.res);
        rl.end_drawing();
    }
}
