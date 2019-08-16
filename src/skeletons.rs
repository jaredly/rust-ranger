use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::HashMap, fs::File};

pub mod component {
    #[derive(Copy, Clone, PartialEq)]
    pub enum Facing {
        Left,
        Right,
    }

    impl Facing {
        fn for_velocity(self, v: &nphysics2d::algebra::Velocity2<f32>) -> Self {
            if v.linear.x.abs() < 0.01 {
                self
            } else if v.linear.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            }
        }
    }

    #[derive(Copy, Clone, PartialEq)]
    pub enum Action {
        Walk,
        Stand,
        Jump,
    }

    #[derive(Copy, Clone, PartialEq)]
    pub enum ArmAction {
        None,
        Throw(na::Vector2<f32>),
        Bow(na::Vector2<f32>),
    }

    impl Action {
        fn for_velocity(v: &nphysics2d::algebra::Velocity2<f32>) -> Self {
            if v.linear.y.abs() < 0.01 {
                Action::Jump
            } else if v.linear.x == 0.0 {
                Action::Stand
            } else {
                Action::Walk
            }
        }
    }

    use specs::prelude::*;
    #[derive(Component)]
    pub struct Skeleton {
        pub name: String,
        pub facing: Facing,
        pub action: Action,
        pub timer: f32,
    }

    impl Skeleton {
        pub fn new(name: &str) -> Self {
            Skeleton {
                name: name.to_owned(),
                facing: Facing::Left,
                action: Action::Stand,
                timer: 0.0,
            }
        }

        pub fn face(&mut self, facing: Facing) {
            self.facing = facing;
        }
    }

    pub struct SkeletonSys;
    use crate::basics::{Body, Collider};

    impl<'a> System<'a> for SkeletonSys {
        type SystemData = (
            Read<'a, crate::basics::Tick>,
            ReadExpect<'a, crate::basics::PhysicsWorld<f32>>,
            ReadStorage<'a, Body>,
            WriteStorage<'a, Skeleton>,
        );

        fn run(&mut self, (tick, physics_world, bodies, mut skeletons): Self::SystemData) {
            for (body, skeleton) in (&bodies, &mut skeletons).join() {
                let v = physics_world
                    .rigid_body(body.0)
                    .unwrap()
                    .part(0)
                    .unwrap()
                    .velocity();
                // skeleton.face(skeleton.facing.for_velocity(&v));
                let new_action = Action::for_velocity(&v);
                if new_action != skeleton.action {
                    skeleton.action = new_action;
                    skeleton.timer = 0.0;
                } else {
                    skeleton.timer += tick.0.as_micros() as f32 / 1000.0;
                }
            }
        }
    }
}

// impl Animatable for na::Vector2<f32> {
//     fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self {
//         na::Vector2::new(
//             Animatable::sin(center.x, frequency, amplitude.x, offset),
//             Animatable::sin(center.y, frequency, amplitude.y, offset),
//         )
//     }
//     fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self {
//         na::Vector2::new(
//             f32::linear(from.x, to.x, speed, offset),
//             f32::linear(from.y, to.y, speed, offset),
//         )
//     }
// }
use crate::scripting::{Animated, Fns, Shared};

#[derive(Debug, Deserialize)]
pub struct Bone {
    pub sprite: String,
    pub offset: (Animated<f32>, Animated<f32>),
    pub pivot_offset: (Animated<f32>, Animated<f32>),
    pub scale: Animated<f32>,
    pub rotation: Animated<f32>,
}

#[derive(Debug, Deserialize)]
pub enum Shape {
    Capsule { width: f32, height: f32 },
    Ball { radius: f32 },
}

#[derive(Debug, Deserialize)]
pub struct Skeleton {
    pub shape: Shape,
    pub scale: Animated<f32>,
    pub offset: (Animated<f32>, Animated<f32>),
    pub bones: Vec<Bone>,
}

#[derive(Debug, Deserialize)]
pub struct Skeletons(pub Fns, pub Shared, pub HashMap<String, Skeleton>);

pub fn read(path: &str) -> Result<Skeletons, ron::de::Error> {
    let f = File::open(path).expect("Failed opening file");
    from_reader(f)
}

pub mod draw {
    use super::*;
    impl Skeletons {
        pub fn draw(
            &self,
            state: &component::Skeleton,
            rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) -> Result<(), crate::scripting::EvalErr> {
            let sk = self.2.get(&state.name).unwrap();
            sk.draw(
                &state, &self.1, &self.0, rd, &sheet, velocity, position, rotation, scale,
            )
        }
    }

    impl Skeleton {
        pub fn draw(
            &self,
            state: &component::Skeleton,
            shared: &Shared,
            fns: &Fns,
            rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) -> Result<(), crate::scripting::EvalErr> {
            let vbls = crate::scripting::Vbls {
                time: state.timer,
                vel: velocity.linear,
            };
            let args = vec![];
            let ctx = crate::scripting::Context { vbls, shared, fns };
            for bone in &self.bones {
                let local_scale = self.scale.eval(&ctx, &args)? * bone.scale.eval(&ctx, &args)?;
                let offset = position
                    + na::Vector2::new(
                        self.offset.0.eval(&ctx, &args)? * local_scale,
                        self.offset.1.eval(&ctx, &args)? * local_scale,
                    )
                    + na::Vector2::new(
                        bone.offset.0.eval(&ctx, &args)? * local_scale,
                        bone.offset.1.eval(&ctx, &args)? * local_scale,
                    );
                sheet.draw(
                    rd,
                    &bone.sprite,
                    (offset.x, offset.y),
                    (
                        bone.pivot_offset.0.eval(&ctx, &args)?
                            * if state.facing == component::Facing::Right {
                                -1.0
                            } else {
                                1.0
                            },
                        bone.pivot_offset.1.eval(&ctx, &args)?,
                    ),
                    rotation + bone.rotation.eval(&ctx, &args)?,
                    scale * local_scale,
                    state.facing == component::Facing::Right,
                )
            }
            Ok(())
        }
    }
}

pub use draw::*;
