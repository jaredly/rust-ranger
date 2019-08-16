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

pub trait Animatable {
    fn sin(center: Self, frequency: f32, amplitude: Self, offset: f32) -> Self;
    fn linear(from: Self, to: Self, speed: f32, offset: f32) -> Self;
    fn add(a: Self, b: Self) -> Self;
    fn mul(a: Self, b: Self) -> Self;
    fn abs(a: Self) -> Self;
}

impl Animatable for f32 {
    fn sin(center: f32, frequency: f32, amplitude: f32, offset: f32) -> f32 {
        (offset / frequency * std::f32::consts::PI * 2.0).sin() * amplitude + center
    }

    fn linear(from: f32, to: f32, speed: f32, offset: f32) -> f32 {
        let at = offset % (speed * 2.0);
        if at > speed {
            from + (to - from) * (at - speed) / speed
        } else {
            from + (to - from) * at / speed
        }
    }

    fn add(a: Self, b: Self) -> Self {
        a + b
    }

    fn mul(a: Self, b: Self) -> Self {
        a * b
    }

    fn abs(a: Self) -> Self {
        a.abs()
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
    fn add(a: Self, b: Self) -> Self {
        (Animatable::add(a.0, b.0), Animatable::add(a.1, b.1))
    }
    fn mul(a: Self, b: Self) -> Self {
        (Animatable::mul(a.0, b.0), Animatable::mul(a.1, b.1))
    }
    fn abs(a: Self) -> Self {
        (Animatable::abs(a.0), Animatable::abs(a.1))
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
#[derive(Copy, Clone)]
pub struct Vbls {
    time: f32,
    vel: na::Vector2<f32>,
}

#[derive(Debug, Deserialize)]
pub enum Animated<T: Animatable + na::base::Scalar> {
    Plain(T),
    Mul(Box<Animated<T>>, Box<Animated<T>>),
    Div(Box<Animated<T>>, Box<Animated<T>>),
    PI,
    TAU,
    Add(Box<Animated<T>>, Box<Animated<T>>),
    Abs(Box<Animated<T>>),
    Shared(String),
    Time,
    Vx,
    Vy,
    V,
    Sin(Box<Animated<T>>),
    SinExt {
        center: T,
        frequency: Box<Animated<f32>>,
        amplitude: T,
        offset: Box<Animated<f32>>,
    },
    Linear {
        from: T,
        to: T,
        speed: Box<Animated<f32>>,
        offset: Box<Animated<f32>>,
    },
}

impl Animated<f32> {
    fn eval(&self, vbls: Vbls, shared: &Shared) -> f32 {
        match self {
            Animated::Plain(t) => *t,
            Animated::PI => std::f32::consts::PI,
            Animated::TAU => std::f32::consts::PI * 2.0,
            Animated::Time => vbls.time,
            Animated::Vx => vbls.vel.x,
            Animated::Vy => vbls.vel.y,
            Animated::V => vbls.vel.norm_squared().sqrt(),
            Animated::Mul(a, b) => a.eval(vbls, shared) * b.eval(vbls, shared),
            Animated::Add(a, b) => a.eval(vbls, shared) + b.eval(vbls, shared),
            Animated::Div(a, b) => a.eval(vbls, shared) / b.eval(vbls, shared),
            Animated::Abs(a) => Animatable::abs(a.eval(vbls, shared)),
            Animated::Shared(key) => shared.get(key).unwrap().eval(vbls, shared),
            Animated::Sin(a) => a.eval(vbls, shared).sin(),
            Animated::SinExt {
                center,
                frequency,
                amplitude,
                offset,
            } => Animatable::sin(
                *center,
                frequency.eval(vbls, shared),
                *amplitude,
                (offset.eval(vbls, shared)
                    * frequency.eval(vbls, shared)
                    * std::f32::consts::PI
                    * 2.0)
                    + vbls.time,
            ),
            Animated::Linear {
                from,
                to,
                speed,
                offset,
            } => Animatable::linear(
                *from,
                *to,
                speed.eval(vbls, shared),
                (offset.eval(vbls, shared) * speed.eval(vbls, shared)) + vbls.time,
            ),
        }
    }
}

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

pub type Shared = HashMap<String, Animated<f32>>;

#[derive(Debug, Deserialize)]
pub struct Skeletons(pub Shared, pub HashMap<String, Skeleton>);

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
            shared: &Shared,
            rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) {
            let vbls = Vbls {
                time: state.timer,
                vel: velocity.linear,
            };
            for bone in &self.bones {
                let local_scale = self.scale.eval(vbls, shared) * bone.scale.eval(vbls, shared);
                let offset = position
                    + na::Vector2::new(
                        self.offset.0.eval(vbls, shared) * local_scale,
                        self.offset.1.eval(vbls, shared) * local_scale,
                    )
                    + na::Vector2::new(
                        bone.offset.0.eval(vbls, shared) * local_scale,
                        bone.offset.1.eval(vbls, shared) * local_scale,
                    );
                sheet.draw(
                    rd,
                    &bone.sprite,
                    (offset.x, offset.y),
                    (
                        bone.pivot_offset.0.eval(vbls, shared)
                            * if state.facing == component::Facing::Right {
                                -1.0
                            } else {
                                1.0
                            },
                        bone.pivot_offset.1.eval(vbls, shared),
                    ),
                    rotation + bone.rotation.eval(vbls, shared),
                    scale * local_scale,
                    state.facing == component::Facing::Right,
                )
            }
        }
    }
}

pub use draw::*;
