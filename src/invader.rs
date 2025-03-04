use bevy::prelude::*;
use spacerl::{
    action::{Actor, Npc, Quickness},
    config::TILE_SIZE,
    map::viewshed::Viewshed,
    movement::Position,
    visuals::glyph::Glyph,
};

/// Marker component for invaders
/// TODO: Move this to Escape Pod?
#[derive(Component, Default)]
#[require(Actor, Npc, Viewshed)]
pub struct Invader;

pub struct SpawnInvader {
    pub name: &'static str,
    pub pos: Position,
    pub quickness: i32,
}

impl Command for SpawnInvader {
    fn apply(self, world: &mut World) {
        let _ = world.run_system_cached_with(spawn_invader, self);
    }
}

fn spawn_invader(spawn_actor: In<SpawnInvader>, mut commands: Commands) {
    commands.spawn((
        Name::new(spawn_actor.name),
        Invader,
        Glyph::new('i', Color::srgb(0.8, 0.2, 0.2), None),
        Quickness::new(spawn_actor.quickness),
        spawn_actor.pos,
        spawn_actor.pos.to_transform(TILE_SIZE),
        Visibility::Visible,
        Viewshed::new(5),
    ));
}
