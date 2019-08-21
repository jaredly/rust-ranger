use specs::prelude::*;

use nalgebra::Point2;

extern crate nalgebra as na;

use crate::basics::*;

pub type DrawHandle<'a, 'b> =
    raylib::drawing::RaylibMode2D<'a, raylib::drawing::RaylibDrawHandle<'b, raylib::RaylibHandle>>;

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

use na::Isometry2;
use ncollide2d::shape::{self, Shape};

#[allow(dead_code)]
fn draw_shape(
    rd: &mut DrawHandle,
    offset: Isometry2<f32>,
    shape: &dyn Shape<f32>,
    margin: f32,
    fill: raylib::color::Color,
) {
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        rd.draw_circle_v(
            raylib::math::Vector2::new(offset.translation.x, offset.translation.y),
            s.radius() + margin,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        let size = s.half_extents();
        rd.draw_rectangle_pro(
            raylib::math::Rectangle::new(
                offset.translation.x - size.x - margin,
                offset.translation.y - size.y - margin,
size.x * 2.0 + margin * 2.0, size.y * 2.0 + margin * 2.0
            ),
            raylib::math::Vector2::new(
                0.0,
                0.0,
            ),
            offset.rotation.angle(),
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x;
        let y = offset.translation.y;
        let w= s.radius() * 2.0 + margin * 2.0;
        let h = s.height() + s.radius() * 2.0 + margin * 2.0;
        rd.draw_rectangle_pro(
            raylib::math::Rectangle::new(
                x,
                y,
                w,
                h,
            ),
            raylib::math::Vector2::new(
                w / 2.0,
                h / 2.0
            ),
            offset.rotation.angle() * 180.0 / std::f32::consts::PI,
            fill
        );
    } else if let Some(s) = shape.as_shape::<shape::Compound<f32>>() {
        for &(t, ref s) in s.shapes().iter() {
            draw_shape(rd, offset * t, s.as_ref(), margin, fill);
        }
    }
}

impl<'a> System<'a> for Draw {
    type SystemData = (
        WriteExpect<'a, raylib::RaylibHandle>,
        Entities<'a>,
        Read<'a, crate::ZoomCamera>,
        Read<'a, crate::Camera>,
        ReadExpect<'a, PhysicsWorld<f32>>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Drawable>,
        ReadExpect<'a, crate::sprites::SpriteSheet>,
        ReadStorage<'a, crate::skeletons::component::Skeleton>,
        ReadExpect<'a, crate::skeletons::Skeletons>,
        ReadStorage<'a, crate::player::Player>,
    );

    fn run(
        &mut self,
        (
            mut rl,
            entities,
            zoom_camera,
            camera,
            physics,
            colliders,
            bodies,
            drawables,
            sheet,
            skeletons,
            skeleton_map,
            player,
        ): Self::SystemData,
    ) {
        use raylib::core::drawing::RaylibDraw;

        let pickup_key = rl.is_key_down(raylib::consts::KeyboardKey::KEY_C);

        let mut rd0 = rl.begin_drawing(&self.thread);
        rd0.clear_background(raylib::color::Color::WHITE);
        {
            let mut rd = rd0.begin_mode_2D(zoom_camera.0);

            let offset = -camera.pos;

            for (player, player_collider) in (&player, &colliders).join() {
                // Maybe highlight things that are close enough to reach?
                // if let Some(collider) = physics.collider(player.pickup);
                // for (handle, collider) in physics
                //     .geom
                //     .colliders_in_proximity_of(&physics.colliders, player.pickup)
                //     .unwrap()
                // {
                //     if collider.is_sensor() || handle == player_collider.0 {
                //         continue;
                //     }
                //     let body = physics.rigid_body(collider.body()).unwrap();
                //     if body.is_ground() {
                //         continue;
                //     }
                //     let p = collider.position();
                //     draw_shape(
                //         &mut rd,
                //         Isometry2::from_parts((p.translation.vector + offset).into(), p.rotation),
                //         collider.shape(),
                //         0.1,
                //         raylib::color::Color::new(255, 255, 100, 255),
                //     );
                // }

                if let Some((collider_handle, _entity, _to_vec)) =
                    player.closest_pickupable_entity(&physics, player_collider.0)
                {
                    let collider = physics.collider(collider_handle).unwrap();
                    let p = collider.position();
                    draw_shape(
                        &mut rd,
                        Isometry2::from_parts((p.translation.vector + offset).into(), p.rotation),
                        collider.shape(),
                        0.04,
                        raylib::color::Color::new(255, 255, 100, 200),
                    );
                }
            }

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
                }
            }

            for (entity, collider, skeleton, body) in
                (&entities, &colliders, &skeletons, &bodies).join()
            {
                if let Some(collider) = physics.collider(collider.0) {
                    let v = physics
                        .rigid_body(body.0)
                        .unwrap()
                        .part(0)
                        .unwrap()
                        .velocity();
                    let p = collider.position().translation.vector + offset;
                    let r = collider.position().rotation.angle() * 180.0 / std::f32::consts::PI;
                    match skeleton_map.draw(
                        &skeleton,
                        &mut rd,
                        &sheet,
                        v,
                        p.into(),
                        r,
                        1.0,
                    ) {
                        Ok(()) => (),
                        Err(err) => println!("Failed to draw! Scripting error {:?}", err),
                    };
                }
            }
        }
        rd0.draw_fps(5, 5);
    }
}
