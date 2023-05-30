use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::{
    prelude::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};

// `InspectorOptions` are completely optional
#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    base_thrust: f32,
    rudder_turn_anount: f32,
    boat_half_length: f32,
    friction_linear_term: f32,
    friction_square_term: f32,
    rotational_friction: f32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            base_thrust: 10.0,
            rudder_turn_anount: 0.01,
            boat_half_length: 1.0,
            friction_linear_term: 0.005,
            friction_square_term: 0.000001,
            rotational_friction: 5.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Configuration>() // `ResourceInspectorPlugin` won't initialize the resource
        .register_type::<Configuration>() // you need to register your type to display it
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
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

fn input(
    mut ships: Query<&mut Ship>,
    mut rudder: Query<&mut Rudder>,
    inputs: Res<Input<KeyCode>>,
    constants: Res<Configuration>,
) {
    let mut rudder = rudder.get_single_mut().unwrap();
    for mut ship in ships.iter_mut() {
        if inputs.pressed(KeyCode::W) {
            ship.throttle = 1.0;
        } else {
            ship.throttle = 0.0;
        }
        if inputs.pressed(KeyCode::A) {
            rudder.angle -= constants.rudder_turn_anount;
        }
        if inputs.pressed(KeyCode::D) {
            rudder.angle += constants.rudder_turn_anount;
        }
    }
}

fn update_velocity(
    mut query: Query<&mut Ship>,
    rudders: Query<&Rudder>,
    time: Res<Time>,
    constants: Res<Configuration>,
) {
    let d_t = time.delta_seconds();
    let rudder = rudders.get_single().unwrap();
    for mut ship in query.iter_mut() {
        let thrust_angle = ship.rotation + rudder.angle + PI / 2.0 + PI;
        let force = Vec2::new(
            -constants.base_thrust * thrust_angle.sin(),
            constants.base_thrust * thrust_angle.cos(),
        ) * ship.throttle;
        ship.velocity += force * d_t;

        let torque = -rudder.angle.sin()
            * constants.boat_half_length
            * constants.base_thrust
            * ship.throttle;
        ship.omega += torque * d_t;
    }
}

fn update_position(mut t: Query<(&mut Transform, &Ship)>, time: Res<Time>) {
    for (mut trans, ship) in t.iter_mut() {
        trans.translation.x += ship.velocity.x * time.delta_seconds();
        trans.translation.y += ship.velocity.y * time.delta_seconds();
        trans.rotate_axis(Vec3::Z, ship.omega * time.delta_seconds())
    }
}

fn friction(mut ships: Query<&mut Ship>, time: Res<Time>, constants: Res<Configuration>) {
    for mut ship in ships.iter_mut() {
        let mag = ship.velocity.abs();
        let friction = ship.velocity
            * (constants.friction_linear_term * mag + constants.friction_square_term * mag * mag)
            * time.delta_seconds();
        ship.velocity -= friction;

        let rot_friction = ship.omega * constants.rotational_friction * time.delta_seconds();
        ship.omega -= rot_friction;
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
    omega: f32,
}

impl Default for Ship {
    fn default() -> Self {
        Ship {
            throttle: 0.0,
            /// angles are measured in radians where 0.0 point right
            /// and increasing values move counter-clockwise
            rotation: PI / 2.0,
            velocity: Vec2::default(),
            omega: 0.0,
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
