pub mod ai;
pub mod spawn;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Add sub-plugins
    app.add_plugins(ai::plugin);
    app.add_plugins(spawn::plugin);
}
