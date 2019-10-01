use ron::de::from_reader;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs::File};

pub mod component {
    use serde::{Serialize, Deserialize};

    #[derive(Copy, Clone, PartialEq, Serialize)]
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

    #[derive(Copy, Clone, PartialEq, Serialize)]
    pub enum Action {
        Walk,
        Stand,
        Jump,
    }

    #[derive(Copy, Clone, PartialEq, Serialize)]
    pub enum ArmAction {
        None,
        Throw(na::Vector2<f32>),
        Bow(na::Vector2<f32>),
    }

    impl Action {
        fn for_velocity(v: &nphysics2d::algebra::Velocity2<f32>) -> Self {
            if v.linear.y.abs() > 0.01 {
                Action::Jump
            } else if v.linear.x == 0.0 {
                Action::Stand
            } else {
                Action::Walk
            }
        }
    }

    use specs::prelude::*;
    #[derive(Component, Serialize)]
    pub struct Skeleton {
        pub name: String,
        pub facing: Facing,
        pub action: Action,
        pub action_timer: Option<(Action, f32)>,
        pub pointing: Option<na::Vector2<f32>>,
        pub arm_action: ArmAction,
        pub timer: f32,
    }

    impl Skeleton {
        pub fn new(name: &str) -> Self {
            Skeleton {
                name: name.to_owned(),
                facing: Facing::Left,
                action: Action::Stand,
                action_timer: None,
                pointing: None,
                arm_action: ArmAction::None,
                timer: 0.0,
            }
        }

        pub fn is_standing(&self) -> bool {
            self.action == Action::Stand
        }

        pub fn face(&mut self, facing: Facing) {
            self.facing = facing;
        }

        pub fn set_action(&mut self, action: Action) {
            if action != self.action {
                self.action = action;
                self.timer = 0.0;
            }
        }
    }

    pub struct SkeletonSys;
    use crate::basics::Body;

    impl<'a> System<'a> for SkeletonSys {
        type SystemData = (
            Read<'a, crate::basics::Tick>,
            ReadExpect<'a, crate::basics::PhysicsWorld<f32>>,
            ReadStorage<'a, Body>,
            WriteStorage<'a, Skeleton>,
        );

        fn run(&mut self, (tick, physics_world, bodies, mut skeletons): Self::SystemData) {
            let tick = tick.0.as_micros() as f32 / 1000.0;
            for (body, skeleton) in (&bodies, &mut skeletons).join() {
                // let v = physics_world
                //     .rigid_body(body.0)
                //     .unwrap()
                //     .part(0)
                //     .unwrap()
                //     .velocity();
                // skeleton.face(skeleton.facing.for_velocity(&v));
                // let new_action = Action::for_velocity(&v);
                // if new_action != skeleton.action {
                //     match skeleton.action_timer {
                //         Some((n, t)) if n == new_action => {
                //             if t > 50.0 {
                //                 skeleton.action = new_action;
                //                 skeleton.timer = 0.0;
                //                 skeleton.action_timer = None;
                //                 continue;
                //             } else {
                //                 skeleton.action_timer = Some((n, t + tick));
                //             }
                //         }
                //         _ => skeleton.action_timer = Some((new_action, 0.0)),
                //     }
                // } else {
                //     skeleton.action_timer = None;
                // }
                skeleton.timer += tick;
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
use crate::scripting;
use scripting::{Animated, Bool, Fns, Shared, Simple};

fn if_facing_right() -> Bool<f32> {
    Bool::StrEq {
        key: "facing".into(),
        val: "right".into(),
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Bone {
    pub sprite: String,
    #[serde(default = "Animated::origin")]
    pub offset: (Animated<f32>, Animated<f32>),
    #[serde(default = "Animated::origin")]
    pub pivot_offset: (Animated<f32>, Animated<f32>),
    #[serde(default = "if_facing_right")]
    pub flip: Bool<f32>,
    #[serde(default = "Animated::one")]
    pub scale: Animated<f32>,
    #[serde(default = "Animated::zero")]
    pub rotation: Animated<f32>,
}

pub mod new {
    use super::*;

    fn one() -> f32 { 1.0 }
    fn zero() -> f32 { 1.0 }

    #[derive(Debug, Deserialize, Clone, PartialEq)]
    pub struct Bone {
        pub sprite: String,
        // #[serde(default = "Animated::origin")]
        pub offset: (f32, f32),
        // #[serde(default = "Animated::origin")]
        pub pivot_offset: (f32, f32),
        // #[serde(default = "if_facing_right")]
        pub flip: bool,
        #[serde(default = "one")]
        pub scale: f32,
        #[serde(default = "zero")]
        pub rotation: f32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct Skeleton {
        pub shape: Shape,
        pub scale: f32,
        pub offset: (f32, f32),
        pub bones: Vec<Bone>,
    }

    impl Default for Skeleton {
        fn default() -> Self {
            Skeleton {
                shape: Shape::Capsule { width: 1.0, height: 1.0 },
                scale: 1.0,
                offset: (0.0, 0.0),
                bones: vec![]
            }
        }
    }

}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Shape {
    Capsule { width: f32, height: f32 },
    Ball { radius: f32 },
}

#[derive(Debug, Deserialize)]
pub struct OldSkeleton {
    pub shape: Shape,
    #[serde(default = "Animated::one")]
    pub scale: Animated<f32>,
    #[serde(default = "Animated::origin")]
    pub offset: (Animated<f32>, Animated<f32>),
    pub bones: Vec<Simple<Bone>>,
}

#[derive(Debug, Deserialize)]
pub struct OldSkeletons {
    pub fns: Fns,
    pub shared: Shared,
    pub shared_bones: HashMap<String, Simple<Bone>>,
    pub skeletons: HashMap<String, OldSkeleton>,
}

pub struct Skeletons {
    pub old: OldSkeletons,
    pub new: libretto::Scope,
}

pub fn read(path: &str, new_path: &str) -> Result<Skeletons, ron::de::Error> {
    let f = std::fs::read_to_string(new_path).expect("Failed opening file");
    let new = libretto::eval_file(&f).unwrap();
    let f_old = File::open(path).expect("Failed opening file");
    Ok(Skeletons {
        old: from_reader::<_, OldSkeletons>(f_old)?,
        new: new,
    })
}

pub mod draw {
    use super::*;
    impl Skeletons {
        pub fn draw(
            &self,
            state: &component::Skeleton,
            rd: &mut crate::draw::DrawHandle,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) -> Result<(), crate::scripting::EvalErr> {
            let sk = self.old.skeletons.get(&state.name).unwrap();
            sk.draw(
                // &self,
                &state,
                &self.old.shared,
                &self.old.shared_bones,
                &self.old.fns,
                rd,
                &sheet,
                velocity,
                position,
                rotation,
                scale,
            )
        }
    }

    impl OldSkeleton {
        pub fn draw_new(
            &self,
            state: &component::Skeleton,
            shared: &Shared,
            shared_bones: &HashMap<String, Simple<Bone>>,
            fns: &Fns,
            rd: &mut crate::draw::DrawHandle,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) -> Result<(), libretto::Error> {
            Ok(())
        }

        pub fn draw(
            &self,
            // skeletons: &Skeletons,
            state: &component::Skeleton,
            shared: &Shared,
            shared_bones: &HashMap<String, Simple<Bone>>,
            fns: &Fns,
            rd: &mut crate::draw::DrawHandle,
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
            let mut floats = HashMap::new();
            let mut strings = HashMap::new();

            strings.insert(
                "arm_action".into(),
                match state.arm_action {
                    component::ArmAction::None => "none".to_owned(),
                    component::ArmAction::Throw(v) => {
                        floats.insert("throw_vx".into(), v.x);
                        floats.insert("throw_vy".into(), v.y);
                        floats.insert("throw_mag".into(), v.norm_squared().sqrt());
                        floats.insert("throw_theta".into(), v.y.atan2(v.x));
                        "throw".into()
                    }
                    component::ArmAction::Bow(v) => {
                        floats.insert("throw_vx".into(), v.x);
                        floats.insert("throw_vy".into(), v.y);
                        floats.insert("throw_mag".into(), v.norm_squared().sqrt());
                        floats.insert("throw_theta".into(), v.y.atan2(v.x));
                        "bow".into()
                    }
                },
            );

            if let Some(v) = state.pointing {
                floats.insert("point_vx".into(), v.x);
                floats.insert("point_vy".into(), v.y);
                floats.insert("point_mag".into(), v.norm_squared().sqrt());
                floats.insert("point_theta".into(), v.y.atan2(v.x));
            }

            strings.insert(
                "facing".into(),
                match state.facing {
                    component::Facing::Left => "left".into(),
                    component::Facing::Right => "right".into(),
                },
            );
            strings.insert(
                "action".into(),
                match state.action {
                    component::Action::Jump => "jump".into(),
                    component::Action::Walk => "walk".into(),
                    component::Action::Stand => "stand".into(),
                },
            );
            let ctx = crate::scripting::Context {
                vbls,
                shared,
                floats,
                fns,
                strings,
            };
            let simples = crate::scripting::SimpleContext {
                shared: shared_bones,
            };
            for bone in &self.bones {
                let bone = match bone.eval(&ctx, &simples, &args)? {
                    None => continue,
                    Some(bone) => bone,
                };
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
                let flip = bone.flip.eval(&ctx, &args)?;
                sheet.draw(
                    rd,
                    &bone.sprite,
                    (offset.x, offset.y),
                    (
                        bone.pivot_offset.0.eval(&ctx, &args)? * if flip { -1.0 } else { 1.0 },
                        bone.pivot_offset.1.eval(&ctx, &args)?,
                    ),
                    rotation + bone.rotation.eval(&ctx, &args)?,
                    scale * local_scale,
                    flip,
                )
            }
            Ok(())
        }
    }
}

pub use draw::*;
