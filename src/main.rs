use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

use escape_pod::npc::NpcPlugin;
use escape_pod::npc::invader::spawn::SpawnInvader;
use spacerl::SpaceRLPlugin;
use spacerl::camera::follow::FollowEntity;
use spacerl::config::TILE_SIZE;
use spacerl::health::{Damage, DamageType};
use spacerl::inventory::Inventory;
use spacerl::items::Item;
use spacerl::items::equip::Equippable;
use spacerl::items::weapons::{Weapon, WeaponType};
use spacerl::map;
use spacerl::movement::Position;
use spacerl::player::spawn::SpawnPlayer;
use spacerl::states::{AppState, finish_startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..Default::default()
        }))
        .add_plugins(SpaceRLPlugin)
        // Escape Pod specific plugins
        .add_plugins(NpcPlugin)
        .add_systems(OnEnter(AppState::Startup), (setup, finish_startup).chain())
        .run();
}

/// Setup the game by spawning the camera and NPCs
fn setup(mut commands: Commands) {
    // Generate map
    // let map = map::mapgen::generate_random_map(
    //     map::MapGridSize(TILE_SIZE),
    //     (Position::new(-11, -11), Position::new(11, 11)),
    // );
    let map = map::mapgen::generate_viewshed_test_map(map::components::MapGridSize(TILE_SIZE));

    // Spawn Weapon for player
    let weapon = commands
        .spawn((
            Name::new("Pipe Wrench"),
            Item,
            Equippable,
            Weapon,
            WeaponType::Melee,
            Damage(10),
            DamageType::Kinetic,
        ))
        .id();

    // Spawn player
    let pos = Position::new(0, 0);
    // Shift position if it is on a wall
    let pos = map.get_nearest_walkable_tile(&pos).unwrap_or(pos);
    let inventory = Inventory::new(vec![weapon]);

    commands.queue(SpawnPlayer {
        position: pos,
        inventory,
    });

    // Spawn camera
    // commands.spawn((Camera2d, FollowEntity(Some(player_entity))));
    commands.spawn((Camera2d, FollowEntity(None)));

    let npcs = [
        // (Name, (x, y), (r, g, b))
        ("Kristín", (-2, 2)),
        ("Damir", (-1, -2)),
        ("Elwyn", (3, 0)),
    ];

    for (name, (x, y)) in npcs.iter() {
        let pos = Position::new(*x, *y);
        // Shift position if it is on wall
        let pos = map.get_nearest_walkable_tile(&pos).unwrap_or(pos);
        let inventory = Inventory::new(vec![weapon]);

        commands.queue(SpawnInvader {
            name,
            pos,
            quickness: 100,
            inventory,
        });
    }

    // Spawn Map (and create Map entity)
    commands.queue(map::spawn::SpawnMap { map });
}
