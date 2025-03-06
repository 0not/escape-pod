use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();
        app.register_type::<Damage>();
        app.register_type::<DamageType>();
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Debug, Default, Reflect)]
#[require(DamageType)]
pub struct Damage(pub i32);

#[derive(Component, Debug, Default, Reflect)]
#[non_exhaustive]
pub enum DamageType {
    #[default]
    Kinetic,
    Thermal,
    Explosive,
    Photic,
    Electric,
}
