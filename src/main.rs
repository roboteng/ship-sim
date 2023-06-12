use std::io::Write;
use std::{
    f32::consts::PI,
    fs::{read_to_string, OpenOptions},
};

use bevy::prelude::*;
use bevy_inspector_egui::{
    prelude::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "config.yaml";
const TWO_PI: f32 = 2.0 * PI;

// `InspectorOptions` are completely optional
#[derive(Reflect, Resource, InspectorOptions, Serialize, Deserialize, Debug)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    base_thrust: f32,
    rudder_turn_amount: RadiansPerSec,
    boat_half_length: f32,
    friction_linear_term: f32,
    friction_square_term: f32,
    rotational_friction: f32,
    #[serde(skip_serializing)]
    save_config: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            base_thrust: 10.0,
            rudder_turn_amount: 0.01,
            boat_half_length: 1.0,
            friction_linear_term: 0.005,
            friction_square_term: 0.000001,
            rotational_friction: 5.0,
            save_config: false,
        }
    }
}

fn main() {
    let config_s = read_to_string(CONFIG_PATH).unwrap_or_default();
    let config = serde_yaml::from_str::<Configuration>(&config_s).unwrap_or_default();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .register_type::<Configuration>() // you need to register your type to display it
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
        .add_startup_system(startup)
        .add_system(input)
        .add_system(update_velocity)
        .add_system(friction)
        .add_system(draw_ship)
        .add_system(draw_rudder)
        .add_system(save_config)
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
            rudder.angle -= constants.rudder_turn_amount;
        }
        if inputs.pressed(KeyCode::D) {
            rudder.angle += constants.rudder_turn_amount;
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
        let constants = constants.as_ref();
        let ship = ship.as_mut();
        ship.update(constants, d_t, rudder);
    }
}

fn draw_ship(mut t: Query<(&mut Transform, &Ship)>) {
    for (mut trans, ship) in t.iter_mut() {
        trans.translation.x = ship.pos.x;
        trans.translation.y = ship.pos.y;
        let mut t = Transform::IDENTITY.clone();
        t.rotate_axis(Vec3::Z, ship.rotation);
        trans.rotation = t.rotation;
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

fn draw_rudder(mut rudders: Query<(&mut Transform, &Rudder)>) {
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

fn save_config(config: Res<Configuration>) {
    if config.is_changed() && config.save_config {
        fn write(config: &Configuration) -> Result<(), ()> {
            let parsed = serde_yaml::to_string(config).map_err(|_| ())?;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(CONFIG_PATH)
                .map_err(|_| ())?;
            write!(file, "{}", parsed).map_err(|_| ())?;
            Ok(())
        }

        match write(&config) {
            Ok(_) => {}
            Err(_) => eprintln!("Unable to save config.yaml"),
        }
    }
}

/// angles are measured in radians where 0.0 point right
/// and increasing values move counter-clockwise
type Radians = f32;
type RadiansPerSec = f32;

#[derive(Component)]
struct Ship {
    throttle: f32,
    rotation: Radians,
    velocity: Vec2,
    omega: RadiansPerSec,
    pos: Vec2,
}

impl Ship {
    fn update(&mut self, constants: &Configuration, d_t: f32, rudder: &Rudder) {
        let thrust_angle = self.rotation + (rudder.angle);

        let force = Vec2::new(
            -constants.base_thrust * thrust_angle.sin(),
            constants.base_thrust * thrust_angle.cos(),
        ) * self.throttle;
        self.velocity += force * d_t;

        let torque = -rudder.angle.sin()
            * constants.boat_half_length
            * constants.base_thrust
            * self.throttle;
        self.omega += torque * d_t;

        self.pos += self.velocity * d_t;
        self.rotation += self.omega * d_t;
    }
}

impl Default for Ship {
    fn default() -> Self {
        Ship {
            throttle: 0.0,
            rotation: 0.0,
            velocity: Vec2::default(),
            omega: 0.0,
            pos: Vec2::default(),
        }
    }
}

#[derive(Component)]
struct Rudder {
    angle: Radians,
}

impl Default for Rudder {
    fn default() -> Self {
        Rudder { angle: -PI / 2.0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn up() {
        let mut ship = Ship {
            throttle: 1.0,
            rotation: PI / 2.0,
            ..default()
        };
        let config = Configuration::default();
        let rudder = Rudder::default();
        ship.update(&config, 1. / 60., &rudder);

        assert!(ship.velocity.length() > 0.0);
        assert_float_absolute_eq!(Vec2::X.angle_between(ship.velocity), PI / 2.0);
    }

    #[test]
    fn left() {
        let mut ship = Ship {
            throttle: 1.0,
            rotation: PI,
            velocity: Vec2::default(),
            omega: 0.0,
            pos: Vec2::default(),
        };
        let config = Configuration::default();
        let rudder = Rudder::default();

        ship.update(&config, 1.0 / 60.0, &rudder);

        assert!(ship.velocity.length() > 0.0);

        assert_angles_eq(Vec2::X.angle_between(ship.velocity), PI);
    }

    #[test]
    fn rotate_rudder_straight() {
        let mut ship = Ship {
            throttle: 1.0,
            ..default()
        };
        let config = Configuration::default();
        let rudder = Rudder::default();

        ship.update(&config, 1.0 / 60.0, &rudder);

        assert_float_absolute_eq!(ship.omega, 0.0);
    }

    #[test]
    fn rotate_rudder_left() {
        let mut ship = Ship {
            throttle: 1.0,
            ..default()
        };
        let config = Configuration::default();
        let rudder = Rudder {
            angle: -PI * 3.0 / 4.0,
        };

        ship.update(&config, 1.0 / 60.0, &rudder);

        assert!(ship.omega > 0.0);
    }

    #[test]
    fn rotate_rudder_right() {
        let mut ship = Ship {
            throttle: 1.0,
            ..default()
        };
        let config = Configuration::default();
        let rudder = Rudder { angle: -PI / 4.0 };

        ship.update(&config, 1.0 / 60.0, &rudder);

        assert!(ship.omega < 0.0);
    }

    fn assert_angles_eq(a: f32, b: f32) {
        let epsilon = 1e-6;

        assert!(((a - b + PI).rem_euclid(TWO_PI) - PI).abs() < epsilon);
    }

    mod helper_test {
        use std::panic::catch_unwind;

        use super::*;
        #[test]
        fn zerp_and_zero() {
            assert_angles_eq(0.0, 0.0);
        }

        #[test]
        fn zero_and_half_turn() {
            match catch_unwind(|| {
                assert_angles_eq(0.0, PI);
            }) {
                Ok(()) => panic!("expected an assertion failure, but didn't get one"),
                Err(_) => {}
            }
        }

        #[test]
        fn half_turn_and_many() {
            assert_angles_eq(PI, 9.0 * PI);
        }
    }
}
