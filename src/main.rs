use bevy::prelude::*;

const FORCE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(locations)
        .add_system(update_velocity)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let ship_handle = asset_server.load("ship.png");
    commands.spawn((
        SpriteBundle {
            texture: ship_handle,
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            visibility: Visibility::Visible,
            ..default()
        },
        Ship,
        Velocity::default(),
    ));
}

fn locations(mut locations: Query<&mut Velocity, With<Ship>>, inputs: Res<Input<KeyCode>>) {
    for mut ship in locations.iter_mut() {
        if inputs.pressed(KeyCode::W) {
            ship.0.y += FORCE;
        }
        if inputs.pressed(KeyCode::A) {
            ship.0.x -= FORCE;
        }
        if inputs.pressed(KeyCode::S) {
            ship.0.y -= FORCE;
        }
        if inputs.pressed(KeyCode::D) {
            ship.0.x += FORCE;
        }
    }
}

fn update_velocity(mut t: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut pos, v) in t.iter_mut() {
        pos.translation.x += v.0.x * FORCE * time.delta_seconds();
        pos.translation.y += v.0.y * FORCE * time.delta_seconds();
    }
}

#[derive(Component)]
struct Ship;

#[derive(Component, Default)]
struct Velocity(Vec2);
