use bevy::prelude::*;
use bevy_action_ticker::{ActionTick, ActionTickStatus, ActionTickerPlugin, PreActionTick};

use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use itertools::Itertools;
use space_rogue::action::{Actor, ActorPlugin, Npc, Quickness};
use space_rogue::animation::{active_actor_animation, transform_animation};
use space_rogue::assets::SquareTextFont;
use space_rogue::config::TILE_SIZE;
use space_rogue::debugging::DebugPlugin;
use space_rogue::map::shapes::Line;
use space_rogue::map::viewshed::{Viewshed, mark_dirty_viewsheds, update_viewsheds};
use space_rogue::map::visibility::{
    show_map_tiles, show_visible_entities, update_map_tile_visibility,
};
use space_rogue::map::{self, MapGlyph, MapPlugin};
use space_rogue::movement::gravity::{Velocity, VelocityArrow, apply_velocity};
use space_rogue::movement::{Direction, Position, update_transforms_on_position_change};
use space_rogue::player::{Player, PlayerPlugin};
use space_rogue::states::{AppState, MenuState, finish_loading, finish_startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DebugPlugin)
        .add_plugins(ActionTickerPlugin::default())
        .add_plugins(ActorPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(PlayerPlugin)
        .register_type::<Position>()
        .register_type::<Velocity>()
        .register_type::<ActionTickCounter>()
        .init_resource::<ActionTickCounter>()
        .register_type::<ActionRoundCounter>()
        .init_resource::<ActionRoundCounter>()
        .add_plugins(ResourceInspectorPlugin::<ActionTickCounter>::default())
        .add_plugins(ResourceInspectorPlugin::<ActionRoundCounter>::default())
        .init_state::<AppState>()
        .add_sub_state::<MenuState>()
        .add_systems(
            OnEnter(AppState::Loading),
            (
                space_rogue::assets::setup_tile_font_resource,
                finish_loading,
            )
                .chain(),
        )
        .add_systems(OnEnter(AppState::Startup), (setup, finish_startup).chain())
        .add_systems(OnEnter(AppState::InGame), unpause_action_ticker)
        .add_systems(OnExit(AppState::InGame), pause_action_ticker)
        .add_systems(
            Update,
            (
                camera_follow_entity,
                update_transforms_on_position_change,
                transform_animation,
                active_actor_animation,
                (
                    mark_dirty_viewsheds,
                    update_viewsheds,
                    update_map_tile_visibility,
                    show_map_tiles,
                    show_visible_entities,
                )
                    .chain(),
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, draw_velocity_arrow)
        .add_systems(PreActionTick, (action_round_counter, apply_velocity))
        .add_systems(ActionTick, action_tick_counter)
        .add_systems(OnEnter(AppState::InGame), || info!("Entering InGame state"))
        .run();
}

#[derive(Resource, Default, Reflect)]
pub struct ActionTickCounter(pub u64);

#[derive(Resource, Default, Reflect)]
pub struct ActionRoundCounter(pub u64);

/// System that increments the action tick counter
pub fn action_tick_counter(mut action_tick_counter: ResMut<ActionTickCounter>) {
    action_tick_counter.0 += 1;
}

/// System that increments the action round counter
pub fn action_round_counter(mut action_round_counter: ResMut<ActionRoundCounter>) {
    action_round_counter.0 += 1;
}

pub fn pause_action_ticker(mut action_tick_status: ResMut<ActionTickStatus>) {
    *action_tick_status = ActionTickStatus::Paused;
}

pub fn unpause_action_ticker(mut action_tick_status: ResMut<ActionTickStatus>) {
    *action_tick_status = ActionTickStatus::Ticking;
}

/// Component for camera entity to follow another entity
#[derive(Component, Default, Debug)]
pub struct FollowEntity(pub Option<Entity>);

/// System that forces camera to follow an entity
/// TODO: Could re-parent the camera to the entity instead of moving the camera
pub fn camera_follow_entity(
    mut q_camera: Query<(&mut Transform, &FollowEntity), With<Camera2d>>,
    q_entity: Query<&Transform, Without<Camera2d>>,
) {
    for (mut camera_transform, follow_entity) in q_camera.iter_mut() {
        if let Some(entity) = follow_entity.0 {
            if let Ok(entity_transform) = q_entity.get(entity) {
                camera_transform.translation = entity_transform.translation;
            }
        }
    }
}

fn draw_velocity_arrow(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut q_velocity: Query<(&Position, &Velocity, &mut VelocityArrow), Changed<Position>>,
    text_font: Res<SquareTextFont>,
) {
    for (pos, vel, mut vel_arrow) in q_velocity.iter_mut() {
        gizmos.line_2d(
            pos.to_vec2() * TILE_SIZE,
            (pos.to_vec2() + vel.velocity) * TILE_SIZE,
            Color::srgb(1., 0., 0.),
        );

        // Clear the VelocityArrow
        vel_arrow.positions.clear();

        let direction = vel.velocity; // .normalize();
        let direction = Position::new(direction.x.round() as i32, direction.y.round() as i32);
        // info!("Entity: {:?} | Direction: {:?}", entity, direction);
        for (tile, next_tile) in Line::new(*pos, *pos + direction)
            .with_step(1.)
            .tuple_windows()
        {
            let direction = Direction::try_from(&(next_tile - tile));
            if let Ok(direction) = direction {
                info!(
                    "Tile: {:?}, Next Tile: {:?}, Direction: {:?}",
                    tile, next_tile, direction
                );

                let glyph: char = match direction {
                    Direction::N => '↑',
                    Direction::NE => '↗',
                    Direction::E => '→',
                    Direction::SE => '↘',
                    Direction::S => '↓',
                    Direction::SW => '↙',
                    Direction::W => '←',
                    Direction::NW => '↖',
                };

                // Using next_tile so that the arrow is not drawn over the entity with velocity
                vel_arrow.positions.insert(next_tile, MapGlyph(glyph));
            }
        }

        // Despawn the old arrows children, or create new arrows parent if it doesn't exist
        let arrows = if let Some(entity) = vel_arrow.entity {
            if let Some(mut e) = commands.get_entity(entity) {
                e.try_despawn_descendants();
            }
            entity
        } else {
            let arrows = commands
                .spawn((
                    Name::new("Arrow"),
                    Transform::from_translation(Vec3::ZERO),
                    Visibility::Visible,
                ))
                .id();

            arrows
        };

        vel_arrow.entity = Some(arrows);

        // Spawn the new arrows
        for (pos, glyph) in vel_arrow.positions.iter() {
            if let Some(mut e) = commands.get_entity(arrows) {
                e.with_children(|parent| {
                    parent.spawn((
                        Text2d::new(glyph.0),
                        text_font.0.clone(),
                        *pos,
                        Transform::from_translation(pos.to_translation(TILE_SIZE)),
                    ));
                });
            };
        }
        // let arrows =
        //     .with_children(|parent| {
        //         parent.spawn((Text2d::new(glyph), text_font.0.clone()));
        //     })
        //     .id();
    }
}

/// Setup the game by spawning the camera and NPCs
fn setup(mut commands: Commands, text_font: Res<SquareTextFont>) {
    // Get loaded font
    let text_font = text_font.0.clone();

    // Spawn map
    // let map = map::mapgen::generate_random_map(
    //     map::MapGridSize(TILE_SIZE),
    //     (Position::new(-11, -11), Position::new(11, 11)),
    // );
    let map = map::mapgen::generate_viewshed_test_map(map::MapGridSize(TILE_SIZE));

    for (pos, map_tile_data) in map.map_data.iter() {
        // let translation = pos.to_translation(TILE_SIZE).with_z(-1.);
        commands
            .spawn((
                Name::new(format!("Tile ({}, {})", pos.x, pos.y)),
                map::MapTile,
                map_tile_data.map_tile_type,
                Position::new(pos.x, pos.y),
                Transform::from_translation(pos.to_translation(TILE_SIZE).with_z(-1.)),
                Visibility::Visible,
            ))
            .with_child((map_tile_data.map_tile_style,));
    }

    // Spawn player
    let pos = Position::new(0, 0);
    // Shift position if it is on a wall
    let pos = map.get_nearest_walkable_tile(&pos).unwrap_or(pos);
    let player_entity = commands
        .spawn((
            Name::new("Player"),
            Player,
            Actor,
            Quickness::new(100),
            pos,
            Transform::from_translation(pos.to_translation(TILE_SIZE)),
            Visibility::Visible,
            Viewshed::new(5),
            Velocity::new(-1., 0.),
        ))
        .with_child((
            Text2d::new("@"),
            text_font.clone(),
            TextColor(Color::srgb(0.8, 0.8, 0.)),
        ))
        .id();

    // Spawn camera
    commands.spawn((Camera2d, FollowEntity(Some(player_entity))));

    let npcs = [
        // (Name, (x, y), (r, g, b))
        ("Kristín", (-2, 2), (0.8, 0.2, 0.2)),
        ("Damir", (-1, -2), (0.2, 0.8, 0.2)),
        ("Elwyn", (3, 0), (0.2, 0.2, 0.8)),
    ];

    for (name, (x, y), (r, g, b)) in npcs.iter() {
        let pos = Position::new(*x, *y);
        // Shift position if it is on wall
        let pos = map.get_nearest_walkable_tile(&pos).unwrap_or(pos);

        commands
            .spawn((
                Name::new(*name),
                Actor,
                Npc,
                Quickness::new(100), // Give NPCs 100 AP per tick (the default value)
                pos,
                Transform::from_translation(pos.to_translation(TILE_SIZE)),
                Visibility::Visible,
                Velocity::new(-1., 0.),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new("g"),
                    text_font.clone(),
                    TextColor(Color::srgb(*r, *g, *b)),
                ));
            });
    }

    // Spawn special Actor that applies gravity to entities with [`GravityAffected`] component
    // commands.spawn((Name::new("Gravity"), Actor, GravityActor));

    // Add Map to the world
    // TODO: Add system to keep the map in sync with the entities
    commands.spawn((Name::new("Map"), map));
}
