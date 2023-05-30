use std::f32::consts::PI;

use bevy::prelude::*;

const FORCE: f32 = 1000.0;
const RUDDER_TURN: f32 = 0.01;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(input)
        .add_system(update_velocity)
        .add_system(friction)
        .add_system(update_position)
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
            Ship::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                SpriteBundle {
                    texture: asset_server.load("rudder.png"),
                    transform: Transform::from_translation(Vec3::new(0., -130., 0.))
                        .with_scale(Vec3::splat(0.2)),
                    ..default()
                },
                Rudder { angle: 0. },
            ));
        });
}

fn input(mut ships: Query<&mut Ship>, mut rudder: Query<&mut Rudder>, inputs: Res<Input<KeyCode>>) {
    let mut rudder = rudder.get_single_mut().unwrap();
    for mut ship in ships.iter_mut() {
        if inputs.pressed(KeyCode::W) {
            ship.throttle = 1.0;
        } else {
            ship.throttle = 0.0;
        }
        if inputs.pressed(KeyCode::A) {
            rudder.angle -= RUDDER_TURN;
        }
        if inputs.pressed(KeyCode::D) {
            rudder.angle += RUDDER_TURN;
        }
    }
}

fn update_velocity(mut query: Query<&mut Ship>, rudders: Query<&Rudder>, time: Res<Time>) {
    let d_t = time.delta_seconds();
    let rudder = rudders.get_single().unwrap();
    for mut ship in query.iter_mut() {
        let thrust_angle = ship.rotation + rudder.angle + PI / 2.0 + PI;
        let force =
            Vec2::new(-FORCE * thrust_angle.sin(), FORCE * thrust_angle.cos()) * ship.throttle;
        ship.velocity += force * d_t;
    }
}

fn update_position(mut t: Query<(&mut Transform, &Ship)>, time: Res<Time>) {
    for (mut trans, ship) in t.iter_mut() {
        trans.translation.x += ship.velocity.x * time.delta_seconds();
        trans.translation.y += ship.velocity.y * time.delta_seconds();
    }
}

fn friction(mut ships: Query<&mut Ship>, time: Res<Time>) {
    for mut ship in ships.iter_mut() {
        let mag = ship.velocity.abs();
        let a = 0.5;
        let b = 0.00001;
        let friction = ship.velocity * (a * mag + b * mag * mag) * time.delta_seconds();
        ship.velocity -= friction
    }
}

fn apply_rudder_changes(mut rudders: Query<(&mut Transform, &Rudder)>) {
    for (mut transform, rudder) in rudders.iter_mut() {
        let mut new =
            Transform::from_translation(Vec3::new(0., -130., 0.)).with_scale(Vec3::splat(0.2));
        new.rotate_around(
            Vec3::new(0., -110., 0.0),
            Quat::from_rotation_z(rudder.angle),
        );
        transform.set_if_neq(new);
    }
}

#[derive(Component)]
struct Ship {
    throttle: f32,
    rotation: f32,
    velocity: Vec2,
}

impl Default for Ship {
    fn default() -> Self {
        Ship {
            throttle: 0.0,
            /// angles are measured in radians where 0.0 point right
            /// and increasing values move counter-clockwise
            rotation: PI / 2.0,
            velocity: Vec2::default(),
        }
    }
}

#[derive(Component)]
struct Rudder {
    angle: f32,
}

impl Default for Rudder {
    fn default() -> Self {
        Rudder { angle: -PI / 2.0 }
    }
}
