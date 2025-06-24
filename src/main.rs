mod camera;
mod diagnostics;
mod gizmos;

use crate::camera::ControlCameraPlugin;
use crate::diagnostics::DiagnosticsPlugin;
use crate::gizmos::ControlGizmoPlugin;
use avian3d::prelude::*;
use bevy::color::palettes::css::DARK_GREY;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            DiagnosticsPlugin,
            ControlGizmoPlugin,
            ControlCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (reset_pinball_on_b, reset_table_on_t))
        .add_systems(Update, nudge_table_on_space)
        .run();
}

// Marker component for the pinball
#[derive(Component)]
struct Pinball;

// Marker component for the player camera
#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct PinballTable;

// 0,0 is the top left corner of the playfield

const PLAYFIELD_WIDTH: f32 = 0.51435; // Width of the pinball playfield
const PLAYFIELD_LENGTH: f32 = 1.06068; // Length of the pinball playfield
const PLAYFIELD_CENTER: Vec3 = Vec3::new(PLAYFIELD_WIDTH / 2.0, 0.0, PLAYFIELD_LENGTH / 2.0);
const PLAYFIELD_Y: f32 = 0.07; // Y position of the playfield center

// 27mm pinball diameter
const BALL_DIAMETER: f32 = 0.027;
const BALL_RADIUS: f32 = BALL_DIAMETER / 2.0;
// 80 grams pinball mass
const BALL_MASS: f32 = 0.080;
const BALL_START_POSITION: Vec3 = Vec3::new(PLAYFIELD_WIDTH / 2.0, 0.50, BALL_RADIUS);
const RESTITUTION_COEFFICIENT_WOOD: f32 = 0.2;
const RESTITUTION_COEFFICIENT_BALL: f32 = 0.5;

const TABLE_START_POSITION: Vec3 =
    Vec3::new(PLAYFIELD_WIDTH / 2.0, PLAYFIELD_Y, PLAYFIELD_LENGTH / 2.0);
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Pinball table dimensions
    let thickness = 0.01;
    let back_wall_height = 0.3;
    let wall_height = 0.1;
    let table_height = 0.5;

    // When the table is standing on the floor with equal length legs the playfield is
    // typically tilted at 6.5 degrees.
    // TODO is this correct?
    let playfield_tilt_angle = 6.5_f32.to_radians();

    // Player head is about 70cm above the table, 10 cm in front of the table, centered on the table
    let player_head_position = Vec3::new(PLAYFIELD_WIDTH / 2.0, 0.70, PLAYFIELD_LENGTH + 0.2);

    // Static floor below the table
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(4.0, 0.1, 4.0),
        Restitution::ZERO,
        Friction::new(1.0),
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.1, 4.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(
            DARK_GREY.red,
            DARK_GREY.green,
            DARK_GREY.blue,
        ))),
        Transform::from_xyz(PLAYFIELD_WIDTH / 2.0, -0.50, PLAYFIELD_LENGTH / 2.0),
    ));

    // Playfield
    // TODO we need to raise it so that the bottom part of the pinball is above the floor
    commands
        .spawn((
            PinballTable,
            RigidBody::Dynamic,
            LinearDamping(0.0),
            Collider::cuboid(PLAYFIELD_WIDTH, thickness, PLAYFIELD_LENGTH),
            //Restitution::new(RESTITUTION_COEFFICIENT_WOOD),
            ColliderDensity(1000.0), // Make it denser
            Friction::new(0.3),
            Mesh3d(meshes.add(Cuboid::new(PLAYFIELD_WIDTH, thickness, PLAYFIELD_LENGTH))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 124, 144))),
            Transform::from_translation(TABLE_START_POSITION)
                .with_rotation(Quat::from_rotation_x(playfield_tilt_angle)),
        ))
        .with_children(|parent| {
            parent.spawn((
                LinearDamping(10.0),
                Collider::cuboid(PLAYFIELD_WIDTH, back_wall_height, thickness),
                //Restitution::new(RESTITUTION_COEFFICIENT_WOOD),
                ColliderDensity(1000.0), // Make it denser
                Friction::new(0.3),
                Mesh3d(meshes.add(Cuboid::new(PLAYFIELD_WIDTH, back_wall_height, thickness))),
                MeshMaterial3d(materials.add(Color::WHITE)),
                Transform::from_xyz(0.0, 0.0, -PLAYFIELD_LENGTH / 2.0 - thickness / 2.0),
            ));

            // Front (player side, lower)
            parent.spawn((
                LinearDamping(10.0),
                Collider::cuboid(PLAYFIELD_WIDTH, wall_height, thickness),
                //Restitution::new(RESTITUTION_COEFFICIENT_WOOD),
                ColliderDensity(1000.0), // Make it denser
                Friction::new(0.3),
                Mesh3d(meshes.add(Cuboid::new(PLAYFIELD_WIDTH, wall_height, thickness))),
                MeshMaterial3d(materials.add(Color::WHITE)),
                Transform::from_xyz(0.0, 0.0, PLAYFIELD_LENGTH / 2.0 + thickness / 2.0),
            ));
        });

    // // Left
    // commands.spawn((
    //     RigidBody::Static,
    //     Collider::cuboid(thickness, wall_height, table_length),
    //     Mesh3d(meshes.add(Cuboid::new(thickness, wall_height, table_length))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_xyz(-table_length, wall_height / 2.0 - table_height, 0.0)
    //         .with_rotation(Quat::from_rotation_x(tilt_angle)),
    // ));
    // // Right
    // commands.spawn((
    //     RigidBody::Static,
    //     Collider::cuboid(thickness, wall_height, table_length),
    //     Mesh3d(meshes.add(Cuboid::new(thickness, wall_height, table_length))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_xyz(table_length, wall_height, 0.0)
    //         .with_rotation(Quat::from_rotation_x(tilt_angle)),
    // ));

    // chrome pinball
    commands.spawn((
        Pinball,
        RigidBody::Dynamic,
        Collider::sphere(BALL_RADIUS),
        NoAutoMass,
        AngularDamping(1.0),
        Mass(BALL_MASS),
        //Restitution::new(RESTITUTION_COEFFICIENT_BALL),
        //Friction::new(0.1),
        Mesh3d(meshes.add(Sphere::new(BALL_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 1.0,
            perceptual_roughness: 0.05,
            reflectance: 1.0,
            ..default()
        })),
        Transform::from_translation(BALL_START_POSITION),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            shadow_depth_bias: 0.01,
            intensity: 100_000.0,
            range: 3.0,
            ..default()
        },
        Transform::from_xyz(PLAYFIELD_WIDTH / 2.0, 1.5, table_height / 2.0),
    ));

    // Camera (player) at the front side of the table, looking at the center
    commands.spawn((
        Camera3d::default(),
        PlayerCamera,
        Transform::from_translation(player_head_position).looking_at(PLAYFIELD_CENTER, Dir3::Y),
    ));
}

fn reset_pinball_on_b(
    keyboard: Res<ButtonInput<KeyCode>>,
    ball_query: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity), With<Pinball>>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        for (mut transform, mut linear_velocity, mut angular_velocity) in ball_query {
            transform.translation = BALL_START_POSITION;
            *linear_velocity = LinearVelocity::ZERO;
            *angular_velocity = AngularVelocity::ZERO;
        }
    }
}

fn reset_table_on_t(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (&mut Transform, &mut AngularVelocity, &mut LinearVelocity),
        With<PinballTable>,
    >,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        for (mut transform, mut angular_velicity, mut linear_velocity) in &mut query {
            transform.translation = TABLE_START_POSITION;
            transform.rotation = Quat::from_rotation_x(6.5_f32.to_radians());
            *angular_velicity = AngularVelocity::ZERO;
            *linear_velocity = LinearVelocity::ZERO;
        }
    }
}

fn nudge_table_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<Entity, With<PinballTable>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let nudge_strength = 5.0;
        // move all static bodies up a bit
        for entity in &mut query {
            info!("Nudging table entity: {:?}", entity);
            commands
                .entity(entity)
                .insert(ExternalImpulse::new(-nudge_strength * Vec3::Z));
        }
    }
}
