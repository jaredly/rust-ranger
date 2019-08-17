use specs::prelude::*;

use nalgebra::Point2;

extern crate nalgebra as na;

use crate::basics::*;

#[derive(Component)]
pub enum Drawable {
    Sprite {
        name: String,
        scale: f32,
    },
    Rect {
        color: raylib::color::Color,
        width: f32,
        height: f32,
    },
}

pub struct Draw {
    pub thread: raylib::RaylibThread,
}

pub const WORLD_SCALE: f32 = 100.0;

use na::Isometry2;
use ncollide2d::shape::{self, Shape};
#[allow(dead_code)]
fn draw_shape(
    rd: &mut raylib::drawing::RaylibDrawHandle<raylib::RaylibHandle>,
    offset: Isometry2<f32>,
    shape: &dyn Shape<f32>,
) {
    let fill = raylib::color::Color::new(255, 0, 0, 100);
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        rd.draw_circle_v(
            raylib::math::Vector2::new(
                offset.translation.x * WORLD_SCALE,
                offset.translation.y * WORLD_SCALE,
            ),
            s.radius() * WORLD_SCALE,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        let size = s.half_extents();
        rd.draw_rectangle_v(
            raylib::math::Vector2::new(
                (offset.translation.x - size.x) * WORLD_SCALE,
                (offset.translation.y - size.y) * WORLD_SCALE,
            ),
            raylib::math::Vector2::new(size.x * 2.0 * WORLD_SCALE, size.y * 2.0 * WORLD_SCALE),
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x - s.radius();
        let y = offset.translation.y - s.half_height() - s.radius();
        rd.draw_rectangle_rounded(
            raylib::math::Rectangle::new(
                x * WORLD_SCALE,
                y * WORLD_SCALE,
                s.radius() * 2.0 * WORLD_SCALE,
                (s.height() + s.radius() * 2.0) * WORLD_SCALE,
            ),
            s.radius() * WORLD_SCALE,
            10,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Compound<f32>>() {
        for &(t, ref s) in s.shapes().iter() {
            draw_shape(rd, offset * t, s.as_ref());
        }
    }
}

impl<'a> System<'a> for Draw {
    type SystemData = (
        WriteExpect<'a, raylib::RaylibHandle>,
        ReadExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Drawable>,
        ReadExpect<'a, crate::sprites::SpriteSheet>,
        ReadStorage<'a, crate::skeletons::component::Skeleton>,
        ReadExpect<'a, crate::skeletons::Skeletons>,
    );

    fn run(
        &mut self,
        (mut rl, physics, colliders, bodies, drawables, sheet, skeletons, skeleton_map): Self::SystemData,
    ) {
        use raylib::core::drawing::RaylibDraw;

        let mut rd = rl.begin_drawing(&self.thread);
        rd.clear_background(raylib::color::Color::WHITE);

        for (collider, drawable) in (&colliders, &drawables).join() {
            if let Some(collider) = physics.collider(collider.0) {
                let p = collider.position() * Point2::new(0.0, 0.0);
                let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;

                match drawable {
                    Drawable::Sprite { name, scale } => {
                        sheet.draw(
                            &mut rd,
                            &name,
                            (p.x * WORLD_SCALE, p.y * WORLD_SCALE),
                            (0.0, 0.0),
                            r,
                            scale * WORLD_SCALE,
                            false,
                        );
                    }
                    Drawable::Rect {
                        color,
                        width,
                        height,
                    } => {
                        rd.draw_rectangle_v(
                            raylib::math::Vector2::from((
                                (p.x - width / 2.0) * WORLD_SCALE,
                                (p.y - height / 2.0) * WORLD_SCALE,
                            )),
                            raylib::math::Vector2::from((
                                width * WORLD_SCALE,
                                height * WORLD_SCALE,
                            )),
                            color,
                        );
                    }
                }

                draw_shape(&mut rd, *collider.position(), collider.shape());
            }
        }

        for (collider, skeleton, body) in (&colliders, &skeletons, &bodies).join() {
            if let Some(collider) = physics.collider(collider.0) {
                let v = physics
                    .rigid_body(body.0)
                    .unwrap()
                    .part(0)
                    .unwrap()
                    .velocity();
                let p = collider.position() * Point2::new(0.0, 0.0) * WORLD_SCALE;
                let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;
                match skeleton_map.draw(&skeleton, &mut rd, &sheet, v, p, r, WORLD_SCALE) {
                    Ok(()) => (),
                    Err(err) => println!("Failed to draw! Scripting error {:?}", err),
                };
                draw_shape(&mut rd, *collider.position(), collider.shape());
            }
        }

        rd.draw_fps(5, 5);
    }
}
