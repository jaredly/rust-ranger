use specs::prelude::*;

use nalgebra::Point2;

extern crate nalgebra as na;

use crate::basics::*;

pub type DrawHandle<'a, 'b> =
    raylib::drawing::RaylibMode2D<'a, raylib::drawing::RaylibDrawHandle<'b, raylib::RaylibHandle>>;

// pub type DrawHandle<'a> = raylib::drawing::RaylibDrawHandle<'a, raylib::RaylibHandle>;

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

// pub const WORLD_SCALE: f32 = 1.0;

use na::Isometry2;
use ncollide2d::shape::{self, Shape};
#[allow(dead_code)]
fn draw_shape(rd: &mut DrawHandle, offset: Isometry2<f32>, shape: &dyn Shape<f32>) {
    let fill = raylib::color::Color::new(255, 0, 0, 100);
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        rd.draw_circle_v(
            raylib::math::Vector2::new(offset.translation.x, offset.translation.y),
            s.radius(),
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        let size = s.half_extents();
        rd.draw_rectangle_v(
            raylib::math::Vector2::new(
                offset.translation.x - size.x,
                offset.translation.y - size.y,
            ),
            raylib::math::Vector2::new(size.x * 2.0, size.y * 2.0),
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x - s.radius();
        let y = offset.translation.y - s.half_height() - s.radius();
        rd.draw_rectangle_rounded(
            raylib::math::Rectangle::new(x, y, s.radius() * 2.0, s.height() + s.radius() * 2.0),
            s.radius(),
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
        Read<'a, crate::ZoomCamera>,
        Read<'a, crate::Camera>,
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
        (
            mut rl,
            zoom_camera,
            camera,
            physics,
            colliders,
            bodies,
            drawables,
            sheet,
            skeletons,
            skeleton_map,
        ): Self::SystemData,
    ) {
        use raylib::core::drawing::RaylibDraw;

        let mut rd0 = rl.begin_drawing(&self.thread);
        rd0.clear_background(raylib::color::Color::WHITE);
        {
            let mut rd = rd0.begin_mode_2D(zoom_camera.0);

            let offset = -camera.pos;

            for (collider, drawable) in (&colliders, &drawables).join() {
                if let Some(collider) = physics.collider(collider.0) {
                    let p = collider.position() * Point2::new(0.0, 0.0);
                    let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;

                    match drawable {
                        Drawable::Sprite { name, scale } => {
                            sheet.draw(
                                &mut rd,
                                &name,
                                (p.x + offset.x, p.y + offset.y),
                                (0.0, 0.0),
                                r,
                                *scale,
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
                                    (p.x - width / 2.0) + offset.x,
                                    (p.y - height / 2.0) + offset.y,
                                )),
                                raylib::math::Vector2::from((*width, *height)),
                                color,
                            );
                        }
                    }

                    // draw_shape(&mut rd, *collider.position(), collider.shape());
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
                    let p = collider.position().translation.vector + offset;
                    let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;
                    match skeleton_map.draw(&skeleton, &mut rd, &sheet, v, p.into(), r, 1.0) {
                        Ok(()) => (),
                        Err(err) => println!("Failed to draw! Scripting error {:?}", err),
                    };
                    // draw_shape(&mut rd, *collider.position(), collider.shape());
                }
            }
        }
        rd0.draw_fps(5, 5);
    }
}
