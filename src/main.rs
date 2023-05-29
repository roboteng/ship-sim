use bevy::prelude::*;

const FORCE: f32 = 1.0;
const RUDDER_TURN: f32 = 0.01;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(locations)
        .add_system(update_velocity)
        .add_system(apply_rudder_changes)
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
                Rudder { angle: 0. },
            ));
        });
}

fn locations(
    mut locations: Query<&mut Velocity, With<Ship>>,
    mut rudder: Query<&mut Rudder>,
    inputs: Res<Input<KeyCode>>,
) {
    let mut rudder = rudder.get_single_mut().unwrap();
    for mut ship in locations.iter_mut() {
        if inputs.pressed(KeyCode::W) {
            ship.0.y += FORCE;
        }
        if inputs.pressed(KeyCode::A) {
            rudder.angle -= RUDDER_TURN;
        }
        if inputs.pressed(KeyCode::S) {
            ship.0.y -= FORCE;
        }
        if inputs.pressed(KeyCode::D) {
            rudder.angle += RUDDER_TURN;
        }
    }
}

fn update_velocity(mut t: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut pos, v) in t.iter_mut() {
        pos.translation.x += v.0.x * FORCE * time.delta_seconds();
        pos.translation.y += v.0.y * FORCE * time.delta_seconds();
    }
}

fn apply_rudder_changes(mut rudders: Query<(&mut Transform, &Rudder)>) {
    for (mut transform, rudder) in rudders.iter_mut() {
        // transform.rotate_around(Vec3::splat(0.), Quat::from_rotation_z(rudder.angle));
        transform.rotation = Quat::from_rotation_z(rudder.angle)
    }
}

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Rudder {
    angle: f32,
}

#[derive(Component, Default)]
struct Velocity(Vec2);
