#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

use nalgebra::{Point2, Vector2};
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use nphysics2d::object::{BodyHandle, ColliderDesc, ColliderHandle, RigidBodyDesc};
use nphysics2d::world::World as PhysicsWorld;

use std::sync::Arc;

const BOX_SIZE_WIDTH: f32 = 250.0;
const BOX_SIZE_HEIGHT: f32 = 20.0;
const BALL_RADIUS: f32 = 10.0;

#[derive(Component)]
struct Ground(ColliderHandle);

#[derive(Component)]
struct Body(BodyHandle);

struct PhysicsMove;

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
        ReadStorage<'a, Ground>,
        ReadStorage<'a, Body>,
    );

    fn run(&mut self, (rl, physics, grounds, bodies): Self::SystemData) {
        use specs::Join;

        // Finally the ecs part. Iterate thorugh ground and body and draw them.
        for (ground,) in (&grounds,).join() {
            if let Some(collider) = physics.collider(ground.0) {
                // nphysics stores it's data in translation rotation matrixes.
                // have to multiply with the origin to get the world position.
                let p = collider.position() * Point2::new(0.0, 0.0);
                // In reality, the shape would be part of the resources of the system. We cheat here
                rl.draw_rectangle(
                    p.x as i32,
                    (p.y - BOX_SIZE_HEIGHT) as i32,
                    BOX_SIZE_WIDTH as i32,
                    BOX_SIZE_HEIGHT as i32,
                    raylib::Color::BLACK,
                );
            }
        }

        for (ball,) in (&bodies,).join() {
            if let Some(body) = physics.rigid_body(ball.0) {
                let p = body.position() * Point2::new(0.0, 0.0);
                rl.draw_circle(p.x as i32, p.y as i32, BALL_RADIUS, raylib::Color::RED);
            }
        }
    }
}

fn main() {
    let mut world = World::new();
    let mut physics_world: PhysicsWorld<f32> = PhysicsWorld::new();

    // Set up physics world (the hard part) first
    // Raylib uses top left as 0,0 so down is really increasing
    physics_world.set_gravity(Vector2::new(0.0, 9.81));

    // Create the ground
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(BOX_SIZE_WIDTH, BOX_SIZE_HEIGHT)));

    // Add ground to system
    let ground_handle = ColliderDesc::new(ground_shape)
        .translation(Vector2::new(320.0, 240.0))
        .build(&mut physics_world)
        .handle();

    // Create a couple of balls with actual rigidbodies (only rigidbodies have gravity)

    let ball_shape = ShapeHandle::new(Ball::new(BALL_RADIUS));
    let collider = ColliderDesc::new(ball_shape).density(1.0);
    let mut rigid_body = RigidBodyDesc::new().collider(&collider);

    // Note that for rust the last statement without a semicolon is the return
    let ball_handles: Vec<_> = (0..5)
        .map(|i| {
            let x = 320.0 + i as f32 * ((BALL_RADIUS + collider.get_margin()) * 2.0 + 0.1);
            // We use the rigid_body as a template to insert balls into the world
            rigid_body
                .set_translation(Vector2::new(x, 100.0))
                .build(&mut physics_world)
                .handle()
        })
        .collect();

    // Hard part over, populate the specs world
    // First we register our components and physics_world
    world.add_resource(physics_world);
    world.register::<Ground>();
    world.register::<Body>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(PhysicsMove, "p_move", &[])
        // make the physics system a dependency of drawing
        .with(Draw, "draw", &["p_move"])
        .build();

    // ground
    world.create_entity().with(Ground(ground_handle)).build();
    for handle in ball_handles {
        world.create_entity().with(Body(handle)).build();
    }

    // Now we setup raylib for the drawing

    let w = 640;
    let h = 480;
    // Use an reference counted pointer to share raylib between this thread and the drawing one
    let rl = Arc::new(raylib::init().size(w, h).title("ECS Collision").build());
    rl.set_target_fps(60);
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
