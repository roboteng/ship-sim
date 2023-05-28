use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

const FORCE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(locations)
        .add_system(update_velocity)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
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
