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
fn draw_shape(
    rd: &mut DrawHandle,
    offset: Isometry2<f32>,
    shape: &dyn Shape<f32>,
    margin: f32,
    fill: raylib::color::Color,
) {
    // let fill = raylib::color::Color::new(255, 0, 0, 255);
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        rd.draw_circle_v(
            raylib::math::Vector2::new(offset.translation.x, offset.translation.y),
            s.radius() + margin,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        let size = s.half_extents();
        rd.draw_rectangle_v(
            raylib::math::Vector2::new(
                offset.translation.x - size.x - margin,
                offset.translation.y - size.y - margin,
            ),
            raylib::math::Vector2::new(size.x * 2.0 + margin * 2.0, size.y * 2.0 + margin * 2.0),
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x - s.radius() - margin;
        let y = offset.translation.y - s.half_height() - s.radius() - margin;
        rd.draw_rectangle_rounded(
            raylib::math::Rectangle::new(
                x,
                y,
                s.radius() * 2.0 + margin * 2.0,
                s.height() + s.radius() * 2.0 + margin * 2.0,
            ),
            s.radius() * 2.0 + margin * 2.0,
            30,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Compound<f32>>() {
        for &(t, ref s) in s.shapes().iter() {
            draw_shape(rd, offset * t, s.as_ref(), margin, fill);
        }
    }
}

#[allow(dead_code)]
fn outline_shape(rd: &mut DrawHandle, offset: Isometry2<f32>, shape: &dyn Shape<f32>) {
    let fill = raylib::color::Color::new(255, 0, 255, 100);
    let width = 0.05;
    use raylib::core::drawing::RaylibDraw;
    if let Some(s) = shape.as_shape::<shape::Ball<f32>>() {
        rd.draw_ring(
            raylib::math::Vector2::new(offset.translation.x, offset.translation.y),
            s.radius() - width,
            s.radius(),
            0,
            360,
            10,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Cuboid<f32>>() {
        let size = s.half_extents();
        rd.draw_rectangle_lines_ex(
            raylib::math::Rectangle::new(
                offset.translation.x - size.x,
                offset.translation.y - size.y,
                size.x * 2.0,
                size.y * 2.0,
            ),
            width as i32,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Capsule<f32>>() {
        let x = offset.translation.x - s.radius();
        let y = offset.translation.y - s.half_height() - s.radius();
        rd.draw_rectangle_rounded_lines(
            raylib::math::Rectangle::new(x, y, s.radius() * 2.0, s.height() + s.radius() * 2.0),
            s.radius(),
            10,
            width as i32,
            fill,
        );
    } else if let Some(s) = shape.as_shape::<shape::Compound<f32>>() {
        for &(t, ref s) in s.shapes().iter() {
            outline_shape(rd, offset * t, s.as_ref());
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
                // let collider = physics.collider(player.pickup).unwrap();
                // let p = collider.position();
                // draw_shape(
                //     &mut rd,
                //     Isometry2::from_parts((p.translation.vector + offset).into(), p.rotation),
                //     collider.shape(),
                //     0.0,
                //     raylib::color::Color::new(255, 100, 225, 255),
                // );

                // let collider = physics.collider(player_collider.0).unwrap();
                // let p = collider.position();
                // draw_shape(
                //     &mut rd,
                //     Isometry2::from_parts((p.translation.vector + offset).into(), p.rotation),
                //     collider.shape(),
                //     0.0,
                //     raylib::color::Color::new(100, 100, 225, 255),
                // );

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

                    // draw_shape(&mut rd, *collider.position(), collider.shape(), 0.0, raylib::color::Color::RED);
                }
            }

            for (entity, collider, skeleton, body) in
                (&entities, &colliders, &skeletons, &bodies).join()
            {
                // let pointing = if let Some(player) = player.get(entity) {
                //     if pickup_key {
                //         if let Some((collider_handle, entity, to_vec)) =
                //             player.closest_pickupable_entity(&physics, collider.0)
                //         {
                //             Some(to_vec)
                //         } else {
                //             None
                //         }
                //     } else {
                //         None
                //     }
                // } else {
                //     None
                // };

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
                        // pointing,
                        p.into(),
                        r,
                        1.0,
                    ) {
                        Ok(()) => (),
                        Err(err) => println!("Failed to draw! Scripting error {:?}", err),
                    };
                    // draw_shape(&mut rd, *collider.position(), collider.shape(), 0.0, raylib::color::Color::RED);
                }
            }
        }
        rd0.draw_fps(5, 5);
    }
}
