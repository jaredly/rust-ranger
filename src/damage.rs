use specs::prelude::*;

// Can I just define a block as a normal item, but have it have a flag like "static" or something
#[derive(Component)]
pub struct Damage {
    amount: f32,
};
