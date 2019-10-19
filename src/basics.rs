use specs::prelude::*;

use nalgebra::Vector2;

use nphysics2d::object::{DefaultBodyHandle, DefaultColliderHandle};

extern crate nalgebra as na;

use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

pub type Id = arrayvec::ArrayString<[u8; 64]>;

pub fn id(val: &str) -> Id {
    Id::from(val).unwrap()
}

pub struct Tick(pub std::time::Duration);
impl Default for Tick {
    fn default() -> Self {
        Tick(std::time::Duration::from_secs(0))
    }
}

#[derive(Component)]
pub struct Collider(pub DefaultColliderHandle);

#[derive(Component)]
pub struct Body(pub DefaultBodyHandle);

pub struct PhysicsWorld<N: na::RealField> {
    pub mech: DefaultMechanicalWorld<N>,
    pub geom: DefaultGeometricalWorld<N>,
    pub bodies: DefaultBodySet<N>,
    pub colliders: DefaultColliderSet<N>,
    pub joints: DefaultJointConstraintSet<N>,
    pub forces: DefaultForceGeneratorSet<N>,
}

impl PhysicsWorld<f32> {
    pub fn new() -> Self {
        let mech = DefaultMechanicalWorld::new(Vector2::new(0.0, 9.81));
        let geom = DefaultGeometricalWorld::new();

        let bodies = DefaultBodySet::new();
        let colliders = DefaultColliderSet::new();
        let joints = DefaultJointConstraintSet::new();
        let forces = DefaultForceGeneratorSet::new();
        PhysicsWorld {
            mech,
            geom,
            bodies,
            colliders,
            joints,
            forces,
        }
    }
}

impl<N: na::RealField> PhysicsWorld<N> {
    pub fn step(&mut self) {
        self.mech.step(
            &mut self.geom,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.forces,
        );
    }

    pub fn collider(
        &self,
        handle: DefaultColliderHandle,
    ) -> Option<&nphysics2d::object::Collider<N, DefaultColliderHandle>> {
        self.colliders.get(handle)
    }

    pub fn collider_mut(
        &mut self,
        handle: DefaultColliderHandle,
    ) -> Option<&mut nphysics2d::object::Collider<N, DefaultColliderHandle>> {
        self.colliders.get_mut(handle)
    }

    pub fn rigid_body(
        &self,
        handle: DefaultBodyHandle,
    ) -> Option<&dyn nphysics2d::object::Body<N>> {
        self.bodies.get(handle)
    }

    pub fn rigid_body_mut(
        &mut self,
        handle: DefaultBodyHandle,
    ) -> Option<&mut dyn nphysics2d::object::Body<N>> {
        self.bodies.get_mut(handle)
    }
}
