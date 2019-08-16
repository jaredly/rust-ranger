use ncollide2d::pipeline::object::CollisionGroups;

pub static PLAYER_GROUP: usize = 0;

pub fn default() -> CollisionGroups {
    return CollisionGroups::new();
}
pub fn player() -> CollisionGroups {
    CollisionGroups::new().with_membership(&[PLAYER_GROUP])
}

pub fn memberAllButPlayer() -> CollisionGroups {
    let mut groups = CollisionGroups::new();
    groups.copy_membership(&default());
    groups.modify_membership(PLAYER_GROUP, false);
    groups
}

pub fn collide_all_but_player() -> CollisionGroups {
    let mut groups = memberAllButPlayer();
    groups.modify_blacklist(PLAYER_GROUP, true);
    groups
}
