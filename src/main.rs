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

    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("ship.png"),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                visibility: Visibility::Visible,
                ..default()
            },
            Ship,
            Velocity::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                SpriteBundle {
                    texture: asset_server.load("rudder.png"),
                    transform: Transform::from_translation(Vec3::new(0., -130., 0.))
                        .with_scale(Vec3::splat(0.2)),
                    visibility: Visibility::Visible,
                    ..default()
                },
                Rudder,
            ));
        });
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

#[derive(Component)]
struct Rudder;

#[derive(Component, Default)]
struct Velocity(Vec2);
