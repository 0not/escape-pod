use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_action_ticker::ActionTickerPlugin;

use escape_pod::npc::NpcPlugin;
use escape_pod::npc::invader::spawn::SpawnInvader;
use spacerl::action::ActionPlugin;
use spacerl::animation::AnimationPlugin;
use spacerl::assets::GameAssetsPlugin;
use spacerl::camera::CameraPlugin;
use spacerl::camera::follow::FollowEntity;
use spacerl::config::TILE_SIZE;
use spacerl::debugging::DebugPlugin;
use spacerl::map::components::MapGridSize;
use spacerl::map::{self, MapPlugin, spawn::SpawnMap};
use spacerl::movement::{MovementPlugin, Position};
use spacerl::player::{PlayerPlugin, spawn::SpawnPlayer};
use spacerl::states::{AppState, AppStatePlugin, finish_startup};
use spacerl::visuals::VisualsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..Default::default()
        }))
        .add_plugins(DebugPlugin)
        .add_plugins(AppStatePlugin)
        .add_plugins(GameAssetsPlugin)
        .add_plugins(ActionTickerPlugin::default())
        .add_plugins(ActionPlugin)
        .add_plugins(NpcPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(VisualsPlugin)
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
    let map = map::mapgen::generate_viewshed_test_map(MapGridSize(TILE_SIZE));

    // Spawn player
    let pos = Position::new(0, 0);
    // Shift position if it is on a wall
    let pos = map.get_nearest_walkable_tile(&pos).unwrap_or(pos);
    commands.queue(SpawnPlayer { position: pos });

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
        commands.queue(SpawnInvader {
            name,
            pos,
            quickness: 100,
        });
    }

    // Spawn Map (and create Map entity)
    commands.queue(SpawnMap { map });
}
