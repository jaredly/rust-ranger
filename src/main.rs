#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

use nalgebra::{Point2, Vector2};
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle, Capsule};
use nphysics2d::object::{DefaultBodyHandle, ColliderDesc, DefaultColliderHandle, RigidBodyDesc, Ground, BodyPartHandle};

extern crate nalgebra as na;

use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};


use std::sync::Arc;

const BOX_SIZE_WIDTH: f32 = 7.0;
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
        color: raylib::color::Color,
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

struct Draw {
    thread: raylib::RaylibThread,
}


const world_scale: f32 = 100.0;

use ncollide2d::shape::{self, Shape};
use na::Isometry2;
fn draw_shape(rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>, offset: Isometry2<f32>, shape: &dyn Shape<f32>) {
    let fill = raylib::color::Color::new(255,0,0,100);
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Plane<f32>>() {
        // self.add_plane(window, object, colliders, s, color, out)
    } else if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        // self.add_ball(window, object, colliders, delta, s, color, out)
        rd.draw_circle_v(raylib::math::Vector2::new(offset.translation.x * world_scale, offset.translation.y * world_scale), s.radius() * world_scale, fill);
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        // self.add_box(window, object, colliders, delta, s, color, out)
        let size = s.half_extents();
        rd.draw_rectangle_v(raylib::math::Vector2::new((offset.translation.x - size.x) * world_scale, (offset.translation.y - size.y) * world_scale), raylib::math::Vector2::new(size.x * 2.0 * world_scale, size.y * 2.0 * world_scale), fill);
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x - s.radius();
        let y = offset.translation.y - s.half_height() - s.radius();
        rd.draw_rectangle_rounded(raylib::math::Rectangle::new(x * world_scale, y * world_scale, s.radius() * 2.0 * world_scale, (s.height() + s.radius() * 2.0) * world_scale), s.radius() * world_scale, 10, fill);
        // self.add_capsule(window, object, colliders, delta, s, color, out)
    } else if let Some(s) = shape.as_shape::<shape::Compound<f32>>() {
        // for &(t, ref s) in s.shapes().iter() {
        //     self.add_shape(window, object, colliders, delta * t, s.as_ref(), color, out)
        // }
    }
}

impl<'a> System<'a> for Draw {
    type SystemData = (
        WriteExpect<'a, raylib::RaylibHandle>,
        ReadExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Drawable>,
        ReadExpect<'a, sprites::SpriteSheet>,
    );

    fn run(&mut self, (mut rl, physics, colliders, _bodies, drawables, sheet): Self::SystemData) {
        // use specs::Join;
        use raylib::core::drawing::RaylibDraw;

        let mut rd = rl.begin_drawing(&self.thread);
        rd.clear_background(raylib::color::Color::WHITE);

        for (collider, drawable) in (&colliders, &drawables).join() {
            if let Some(collider) = physics.collider(collider.0) {
                let p = collider.position() * Point2::new(0.0, 0.0);
                let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;


                match drawable {
                    Drawable::Sprite {name, scale} => {
                        sheet.draw(&mut rd, &name, (p.x * world_scale, p.y * world_scale), r, scale * world_scale);
                    },
                    Drawable::Rect { color, width, height } => {
                        rd.draw_rectangle_v(
                            raylib::math::Vector2::from(((p.x - width / 2.0)  * world_scale, (p.y - height / 2.0) * world_scale)),
                            raylib::math::Vector2::from((width  * world_scale, height * world_scale)),
                            *color,
                        );
                    }
                }

                draw_shape(&mut rd, *collider.position(), collider.shape());
            }
        }

        rd.draw_fps(5, 5);
        // rd.end_drawing();
    }
}

#[derive(Component)]
struct Player;

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

        let speed = 0.05;
        let max_speed = 3.0;

        let mut push = Vector2::new(0.0, 0.0);
        if rl.is_key_pressed(KEY_W) {
            push.y -= speed * 10.0;
        }
        if rl.is_key_down(KEY_D) {
            push.x += speed;
        }
        if rl.is_key_down(KEY_A) {
            push.x -= speed;
        }
        if rl.is_key_down(KEY_S) {
            push.y += speed;
        }
        if push.x == 0.0 && push.y == 0.0 {
            return
        }

        for (body, _) in (&body, &player).join() {
            let body = physics.rigid_body_mut(body.0).unwrap();
            let part = body.part(0).unwrap();
            let v = part.velocity().linear;
            if push.x > 0.0 && v.x > max_speed {
                push.x = 0.0;
            }
            if push.x < 0.0 && v.x < -max_speed {
                push.x = 0.0;
            }
            body.apply_force(0, &nphysics2d::algebra::Force2::linear(push), nphysics2d::algebra::ForceType::Impulse, true);
        }
    }
}



fn main() {
    let mut world = World::new();
    let mut physics_world: PhysicsWorld<f32> = PhysicsWorld::new();

    world.register::<Collider>();
    world.register::<Drawable>();
    world.register::<Player>();
    world.register::<Body>();

    // Set up physics world (the hard part) first
    // Raylib uses top left as 0,0 so down is really increasing
    // physics_world.set_gravity(Vector2::new(0.0, 9.81));

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

    {
        let mut body = RigidBodyDesc::new().translation(Vector2::new(3.0, 1.0)).build();
        body.set_rotations_kinematic(true);
        let rb = physics_world.bodies.insert(body);
        let collider = ColliderDesc::new(ShapeHandle::new(Capsule::new(0.3, 0.1))).density(1.0).build(BodyPartHandle(rb, 0));
        let cb = physics_world.colliders.insert(collider);
        world.create_entity()
            .with(Body(rb))
            .with(Player)
            .with(Collider(cb))
            .with(Drawable::Sprite {name: "gnome_head.png".to_owned(), scale: 0.4})
            .build();
    }

    // Create the ground
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(BOX_SIZE_WIDTH / 2.0, BOX_SIZE_HEIGHT / 2.0)));

    // Add ground to system
    let ground_handle = physics_world.bodies.insert(Ground::new());
    let ground_collider = ColliderDesc::new(ground_shape)
        .translation(Vector2::new(3.20, 3.40))
        .build(BodyPartHandle(ground_handle, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    // ground

    // Hard part over, populate the specs world
    // First we register our components and physics_world
    world.create_entity()
        .with(Collider(ground_collider))
        .with(Drawable::Rect { color: raylib::color::Color::BLACK, width: BOX_SIZE_WIDTH, height: BOX_SIZE_HEIGHT})
        .build();

    // Vertical ground
    let ground_shape_v = ShapeHandle::new(Cuboid::new(Vector2::new(BOX_SIZE_HEIGHT / 2.0, BOX_SIZE_WIDTH / 2.0)));
    let ground_handle_v = physics_world.bodies.insert(Ground::new());
    let ground_collider = ColliderDesc::new(ground_shape_v)
        .translation(Vector2::new(0.1, 3.40))
        .build(BodyPartHandle(ground_handle_v, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    // ground

    // Hard part over, populate the specs world
    // First we register our components and physics_world
    world.create_entity()
        .with(Collider(ground_collider))
        .with(Drawable::Rect { color: raylib::color::Color::BLACK, width: BOX_SIZE_HEIGHT, height: BOX_SIZE_HEIGHT})
        .build();

    // Vertical ground
    let ground_shape_v = ShapeHandle::new(Cuboid::new(Vector2::new(BOX_SIZE_HEIGHT / 2.0, BOX_SIZE_WIDTH / 2.0)));
    let ground_handle_v = physics_world.bodies.insert(Ground::new());
    let ground_collider = ColliderDesc::new(ground_shape_v)
        .translation(Vector2::new(5.0, 3.40))
        .build(BodyPartHandle(ground_handle_v, 0));
    let ground_collider = physics_world.colliders.insert(ground_collider);
    // ground

    // Hard part over, populate the specs world
    // First we register our components and physics_world
    world.create_entity()
        .with(Collider(ground_collider))
        .with(Drawable::Rect { color: raylib::color::Color::BLACK, width: BOX_SIZE_HEIGHT, height: BOX_SIZE_HEIGHT})
        .build();

    world.add_resource(physics_world);

    let w = 640;
    let h = 480;
    let (mut rl, thread) = raylib::init().size(w, h).title("Examples").build();

    // Use an reference counted pointer to share raylib between this thread and the drawing one
    rl.set_target_fps(60);

    let mut sprites = sprites::SpriteSheet::new();
    sprites.add(&mut rl, &thread, "assets/spritesheet_items.png", "assets/spritesheet_items.xml");
    sprites.add(&mut rl, &thread, "assets/extras.png", "assets/extras.xml");
    sprites.add(&mut rl, &thread, "assets/spritesheet_characters.png", "assets/spritesheet_characters.xml");
    sprites.add(&mut rl, &thread, "assets/spritesheet_particles.png", "assets/spritesheet_particles.xml");
    sprites.add(&mut rl, &thread, "assets/spritesheet_tiles.png", "assets/spritesheet_tiles.xml");

    let mut dispatcher = DispatcherBuilder::new()
        .with(PlayerSys, "player_move", &[])
        .with(PhysicsMove, "p_move", &["player_move"])
        .with_thread_local(Draw { thread })
        .build();

    // Now we setup raylib for the drawing

    world.add_resource(sprites);

    world.add_resource(rl);

    dispatcher.setup(&mut world.res);

    let mut should_close = false;
    while !window_should_close(&world) && !should_close {
        // rl.begin_drawing();
        // rl.clear_background(raylib::Color::WHITE);
        dispatcher.dispatch(&mut world.res);
        // rl.end_drawing();
    }
}

fn window_should_close(world: &World) -> bool {
    let rl = world.read_resource::<raylib::RaylibHandle>();
    rl.window_should_close()
}