use bevy::app::{App, Plugin};

pub mod invader;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        // Add sub-plugins
        app.add_plugins(invader::plugin);
    }
}
