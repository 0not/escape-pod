use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Weapon>();
    app.register_type::<WeaponType>();
    app.register_type::<Damage>();
    app.register_type::<DamageType>();
}

#[derive(Component, Debug, Default, Reflect)]
#[require(WeaponType, Damage)]
pub struct Weapon;

#[derive(Component, Debug, Default, Reflect)]
pub enum WeaponType {
    #[default]
    Melee,
    Ranged,
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
