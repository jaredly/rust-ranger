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
            if v.linear.x == 0.0 {
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

    impl Action {
        fn for_velocity(v: &nphysics2d::algebra::Velocity2<f32>) -> Self {
            if v.linear.y != 0.0 {
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
                skeleton.face(skeleton.facing.for_velocity(&v));
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

pub trait Animatable {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self;
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self;
}

impl Animatable for f32 {
    fn sin(center: f32, frequency: f32, amplitude: f32, offset: f32) -> f32 {
        (offset / frequency).sin() * amplitude + center
    }

    fn linear(from: f32, to: f32, speed: f32, offset: f32) -> f32 {
        let at = offset % (speed * 2.0);
        if at > speed {
            from + (to - from) * (at - speed) / speed
        } else {
            from + (to - from) * at / speed
        }
    }
}

impl<T: Animatable> Animatable for (T, T) {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self {
        (
            T::sin(center.0, frequency, amplitude.0, offset),
            T::sin(center.1, frequency, amplitude.1, offset),
        )
    }
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self {
        (
            T::linear(from.0, to.0, speed, offset),
            T::linear(from.1, to.1, speed, offset),
        )
    }
}

impl Animatable for na::Vector2<f32> {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self {
        na::Vector2::new(
            Animatable::sin(center.x, frequency, amplitude.x, offset),
            Animatable::sin(center.y, frequency, amplitude.y, offset),
        )
    }
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self {
        na::Vector2::new(
            f32::linear(from.x, to.x, speed, offset),
            f32::linear(from.y, to.y, speed, offset),
        )
    }
}

#[derive(Debug, Deserialize)]
pub enum Animated<T: Animatable + Copy> {
    Plain(T),
    Sin {
        center: T,
        frequency: f32,
        amplitude: T,
        offset: f32,
    },
    Linear {
        from: T,
        to: T,
        speed: f32,
        offset: f32,
    },
}

impl<T: Animatable + Copy> Animated<T> {
    fn eval(&self, timer: f32) -> T {
        match self {
            Animated::Plain(t) => *t,
            Animated::Sin {
                center,
                frequency,
                amplitude,
                offset,
            } => T::sin(*center, *frequency, *amplitude, offset + timer),
            Animated::Linear {
                from,
                to,
                speed,
                offset,
            } => T::linear(*from, *to, *speed, offset + timer),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Bone {
    pub sprite: String,
    pub offset: Animated<(f32, f32)>,
    pub pivot_offset: Animated<(f32, f32)>,
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
    pub offset: Animated<(f32, f32)>,
    pub bones: Vec<Bone>,
}

#[derive(Debug, Deserialize)]
pub struct Skeletons(pub HashMap<String, Skeleton>);

pub fn read(path: &str) -> Result<Skeletons, ron::de::Error> {
    let f = File::open(path).expect("Failed opening file");
    from_reader(f)
}

pub mod draw {
    use super::*;
    impl Skeleton {
        pub fn draw(
            &self,
            state: &component::Skeleton,
            rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>,
            sheet: &crate::sprites::SpriteSheet,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) {
            for bone in &self.bones {
                let (x, y) = bone.offset.eval(state.timer);
                let offset = position + na::Vector2::new(x, y);
                sheet.draw(
                    rd,
                    &bone.sprite,
                    (offset.x, offset.y),
                    bone.pivot_offset.eval(state.timer),
                    rotation + bone.rotation.eval(state.timer),
                    scale * self.scale.eval(state.timer) * bone.scale.eval(state.timer),
                )
            }
        }
    }
}

pub use draw::*;
