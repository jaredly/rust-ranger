use specs::prelude::*;

use nalgebra::Vector2;
use ncollide2d::shape::{Ball, Capsule, Cuboid, ShapeHandle};
use nphysics2d::object::{
  BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultColliderHandle, Ground, RigidBodyDesc,
};

use crate::basics::*;
use crate::draw::Drawable;

#[derive(Component)]
pub struct ArrowLauncher(pub Option<Vector2<f32>>);

pub struct ArrowSys;

impl<'a> System<'a> for ArrowSys {
  type SystemData = (
    Entities<'a>,
    ReadExpect<'a, raylib::RaylibHandle>,
    WriteExpect<'a, PhysicsWorld<f32>>,
    WriteStorage<'a, ArrowLauncher>,
    WriteStorage<'a, Collider>,
    WriteStorage<'a, Body>,
    WriteStorage<'a, Drawable>,
  );

  fn run(
    &mut self,
    (
            entities,
            rl,
            mut physics_world,
            mut arrow,
            mut colliders,
            mut bodies,
            mut drawables,
        ): Self::SystemData,
  ) {
    if let Some((mut arrow, collider)) = (&mut arrow, &colliders).join().next() {
      if rl.is_mouse_button_pressed(raylib::consts::MouseButton::MOUSE_LEFT_BUTTON) {
        let vec = rl.get_mouse_position();
        arrow.0 = Some(Vector2::new(vec.x, vec.y));
      } else if rl.is_mouse_button_released(raylib::consts::MouseButton::MOUSE_LEFT_BUTTON) {
        match arrow.0 {
          None => (),
          Some(start) => {
            let vec = rl.get_mouse_position();
            let end = Vector2::new(vec.x, vec.y);
            if let Some(collider) = physics_world.collider(collider.0) {
              let pos = collider.position().translation;
              // create an arrow

              let vec = (start - end) / crate::draw::WORLD_SCALE * 3.0;
              let v = nphysics2d::algebra::Velocity2::new(vec, 0.0);
              let off = vec.normalize();
              let pos = Vector2::new(pos.x, pos.y) + off * 0.3;
              let rb = RigidBodyDesc::new()
                .translation(pos)
                .set_velocity(v)
                .build();
              // use nphysics2d::object::Body;
              // rb.enable_gravity(false);
              let rb_handle = physics_world.bodies.insert(rb);

              // Build the collider.
              let ball_shape = ShapeHandle::new(Ball::new(0.1));
              let mut material = nphysics2d::material::BasicMaterial::new(0.1, 0.5);
              material.restitution_combine_mode =
                nphysics2d::material::MaterialCombineMode::Multiply;
              let mh = nphysics2d::material::MaterialHandle::new(material);

              let co = ColliderDesc::new(ball_shape.clone())
                .density(1.0)
                .material(mh)
                .build(BodyPartHandle(rb_handle, 0));
              let co_handle = physics_world.colliders.insert(co);

              let entity = entities.create();
              bodies.insert(entity, Body(rb_handle)).unwrap();
              colliders.insert(entity, Collider(co_handle)).unwrap();
              drawables
                .insert(
                  entity,
                  Drawable::Sprite {
                    name: "ore_coal.png".to_owned(),
                    scale: 0.5,
                  },
                )
                .unwrap();
            }
          }
        }
        arrow.0 = None;
      }
    }
  }
}
