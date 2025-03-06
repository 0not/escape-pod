pub mod weapons;

use bevy::prelude::*;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        // Add sub-plugins
        app.add_plugins(weapons::plugin);
    }
}
