use ron::de::from_reader;
use serde::{Deserialize};
use std::{collections::HashMap, fs::File};

pub mod component {
    use serde::{Serialize};

    #[derive(Copy, Clone, PartialEq, Serialize)]
    pub enum Facing {
        Left,
        Right,
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

        fn run(&mut self, (tick, _physics_world, bodies, mut skeletons): Self::SystemData) {
            let tick = tick.0.as_micros() as f32 / 1000.0;
            for (_body, skeleton) in (&bodies, &mut skeletons).join() {
                skeleton.timer += tick;
            }
        }
    }
}

pub mod new {
    use super::*;

    fn one() -> f32 {
        1.0
    }
    fn zero() -> f32 {
        1.0
    }

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
                shape: Shape::Capsule {
                    width: 1.0,
                    height: 1.0,
                },
                scale: 1.0,
                offset: (0.0, 0.0),
                bones: vec![],
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Shape {
    Capsule { width: f32, height: f32 },
    Ball { radius: f32 },
}

pub struct Skeletons {
    pub scope: libretto::Scope,
}

pub fn read(path: &str) -> Result<Skeletons, ron::de::Error> {
    let f = std::fs::read_to_string(path).expect("Failed opening file");
    let scope = libretto::eval_file(&f).unwrap();
    Ok(Skeletons { scope })
}

pub mod draw {
    use super::*;
    impl Skeletons {
        pub fn draw_new(
            &mut self,
            state: &component::Skeleton,
            rd: &mut crate::draw::DrawHandle,
            sheet: &crate::sprites::SpriteSheet,
            velocity: nphysics2d::math::Velocity<f32>,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) -> Result<(), libretto::Error> {
            let sk: new::Skeleton = libretto::call_fn!(
                self.scope,
                &state.name,
                state,
                velocity.linear
            )?;
            sk.draw(
                rd,
                &sheet,
                position,
                rotation,
                scale,
            );
            Ok(())
        }
    }

    impl new::Skeleton {
        pub fn draw(
            &self,
            rd: &mut crate::draw::DrawHandle,
            sheet: &crate::sprites::SpriteSheet,
            position: na::Point2<f32>,
            rotation: f32,
            scale: f32,
        ) {
            for bone in &self.bones {
                let local_scale = self.scale * bone.scale;
                let offset = position
                    + na::Vector2::new(
                        self.offset.0 * local_scale,
                        self.offset.1 * local_scale,
                    )
                    + na::Vector2::new(
                        bone.offset.0 * local_scale,
                        bone.offset.1 * local_scale,
                    );
                let flip = bone.flip;
                sheet.draw(
                    rd,
                    &bone.sprite,
                    (offset.x, offset.y),
                    (
                        bone.pivot_offset.0 * if flip { -1.0 } else { 1.0 },
                        bone.pivot_offset.1,
                    ),
                    rotation + bone.rotation,
                    scale * local_scale,
                    flip,
                )
            }
        }
    }
}

pub use draw::*;
