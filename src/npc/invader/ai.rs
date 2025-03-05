use bevy::prelude::*;
use spacerl::{
    action::{Action, ActiveActor, Actor, ChooseAction},
    map::{Map, viewshed::VisibleActors},
    movement::{Direction, Position},
    player::Player,
};

use super::spawn::Invader;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(ChooseAction, invader_choose_action);
}

/// Let the actor choose their next action
///
/// The actor will randomly choose to move or do something else.
/// If moving, the actor will randomly choose a direction.
/// If not moving, the actor will choose the closest actor to attack or shout at.
///     If the closest actor is within 2 tiles, the actor will attack.
///     If the closest actor is not within 2 tiles, the actor will shout.
/// If no other actors are present, the actor will shout at themselves.
/// NOTE: The above description is outdated.
fn invader_choose_action(
    mut commands: Commands,
    q_active_actor: Query<
        (Entity, &Position, &VisibleActors),
        (
            With<Actor>,
            With<ActiveActor>,
            Without<Player>,
            With<Invader>,
        ),
    >,
    q_other_actors: Query<(Entity, &Position), (With<Actor>, Without<ActiveActor>)>,
    q_map: Query<(&Map,)>,
    q_player: Query<Entity, With<Player>>,
) {
    let player_entity = if let Ok(player_entity) = q_player.get_single() {
        player_entity
    } else {
        warn!("No player entity found in choose_action system");
        return;
    };

    for (active_entity, active_pos, visible_actors) in q_active_actor.iter() {
        // Randomly choose whether to move or do something else
        let will_move = rand::random::<bool>();
        let next_action: Action = if visible_actors.0.contains(&player_entity) {
            // If the actor can see the player, follow the player
            Action::Follow(player_entity)
        } else if will_move {
            let mut direction;
            loop {
                // Randomly choose a direction to move
                direction = match rand::random::<u8>() % 8 {
                    0 => Direction::N,
                    1 => Direction::NE,
                    2 => Direction::E,
                    3 => Direction::SE,
                    4 => Direction::S,
                    5 => Direction::SW,
                    6 => Direction::W,
                    _ => Direction::NW,
                };

                // Check if the new position is valid
                //  1. Check for walkability
                //  2. Check for other actors
                let new_pos = *active_pos + Position::from(&direction);
                let (map,) = q_map.get_single().expect("should have map");

                if let Some(map_tile) = map.map_data.get(&new_pos) {
                    // 1. Can't move there, so try again
                    if !map_tile.map_tile_type.is_walkable() {
                        continue;
                    }
                } else {
                    // 1. Not a valid map tile
                    continue;
                }

                // 2. Can't move into another actor, so try again
                if q_other_actors
                    .iter()
                    .any(|(_, other_pos)| new_pos == *other_pos)
                {
                    info!("Can't move into another actor");
                    continue;
                }

                break;
            }

            Action::Move(direction)
        } else {
            // Get closest actor.
            //   If within 2 tiles, then attack.
            //   If not, then shout.
            let mut closest_distance = f32::INFINITY;
            let mut closest_entity = None;

            for (other_entity, other_pos) in q_other_actors.iter() {
                let distance = active_pos.to_vec2().distance(other_pos.to_vec2());
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_entity = Some(other_entity);
                }
            }

            // Two tiles gives a max distance of sqrt(2^2 + 2^2) (Pythagorean theorem)
            if closest_distance <= (8.0_f32).sqrt() {
                // Shouldn't panic because if `closest_distance` is less than f32::INFINITY,
                // then `closest_entity` is Some.
                let closest_entity = closest_entity.expect("should have closest entity");
                Action::Attack(closest_entity)
            } else if let Some(closest_entity) = closest_entity {
                Action::Shout(closest_entity)
            } else {
                // No other actors to shout at so shout at myself
                Action::Shout(active_entity)
            }
        };

        // Insert the next action so `perform_action` can execute it
        commands.entity(active_entity).try_insert(next_action);
    }
}
