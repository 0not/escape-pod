use bevy::prelude::*;
use spacerl::{
    action::{Actor, Npc, Quickness},
    config::TILE_SIZE,
    inventory::Inventory,
    map::viewshed::Viewshed,
    movement::{OccupiesPosition, Position},
    visuals::glyph::Glyph,
};

pub(super) fn plugin(app: &mut App) {
    // Register types
    app.register_type::<Invader>();
    app.register_type::<SpawnInvader>();
}

/// Marker component for invaders
#[derive(Component, Default, Reflect)]
#[require(Actor, Npc, Viewshed, Inventory)]
pub struct Invader;

#[derive(Reflect)]
pub struct SpawnInvader {
    pub name: &'static str,
    pub pos: Position,
    pub quickness: i32,
    pub inventory: Inventory,
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
        OccupiesPosition::Yes,
        Visibility::Visible,
        Viewshed::new(5),
        spawn_actor.inventory.clone(),
    ));
}
