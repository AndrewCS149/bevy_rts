use std::f32::EPSILON;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    resources::{GameCommands, MouseCoords},
    Destination, Selected, Speed, Unit, UNITS,
};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_unit)
            .add_systems(Update, (set_unit_destination, move_unit));
    }
}

#[derive(Bundle)]
struct UnitBundle {
    pub collider: Collider,
    pub damping: Damping,
    pub external_impulse: ExternalImpulse,
    pub name: Name,
    pub rigid_body: RigidBody,
    pub speed: Speed,
    pub destination: Destination,
    pub unit: Unit,
    pub locked_axis: LockedAxes,
}

impl UnitBundle {
    fn new(speed: f32, size: f32) -> Self {
        Self {
            collider: Collider::cylinder(size, size / 2.0),
            damping: Damping {
                linear_damping: 10.0,
                ..default()
            },
            external_impulse: ExternalImpulse::default(),
            name: Name::new("Unit"),
            rigid_body: RigidBody::Dynamic,
            speed: Speed(speed),
            destination: Destination(None),
            unit: Unit,
            locked_axis: (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
        }
    }
}

fn spawn_unit(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut unit = |size: f32, speed: f32, translation: Vec3| -> (PbrBundle, UnitBundle) {
        (
            PbrBundle {
                mesh: meshes.add(Capsule3d::new(size / 2.0, size)),
                transform: Transform {
                    translation: translation,
                    ..default()
                },
                material: materials.add(Color::DARK_GRAY),
                ..default()
            },
            UnitBundle::new(speed, size),
        )
    };

    let offset_increment = 1.5;
    for row_index in 0..(UNITS / 10) {
        let offset = row_index as f32 * offset_increment;

        for i in (0..(UNITS / 5)).filter(|&i| i % 2 == 0) {
            cmds.spawn(unit(1.0, 60.0, Vec3::new(i as f32, 0.5, offset)));
        }
    }
}

pub fn set_unit_destination(
    mouse_coords: ResMut<MouseCoords>,
    mut unit_q: Query<(&mut Destination, &Transform), With<Selected>>,
    input: Res<ButtonInput<MouseButton>>,
    game_cmds: Res<GameCommands>,
) {
    if !input.just_released(MouseButton::Left) || game_cmds.drag_select {
        return;
    }

    for (mut unit_destination, trans) in unit_q.iter_mut() {
        let mut destination = mouse_coords.global;
        destination.y += trans.scale.y / 2.0; // calculate for entity height
        unit_destination.0 = Some(destination);
        println!("Unit Moving");
    }
}

fn move_unit(
    mut unit_q: Query<(&Transform, &mut ExternalImpulse, &Speed, &mut Destination), With<Unit>>,
    time: Res<Time>,
) {
    for (trans, mut ext_impulse, speed, mut destination) in unit_q.iter_mut() {
        if let Some(new_pos) = destination.0 {
            let distance = new_pos - trans.translation;
            if distance.length_squared() <= (speed.0 * time.delta_seconds()).powi(2) + EPSILON {
                destination.0 = None;
                println!("Unit Stopping");
            } else {
                ext_impulse.impulse += distance.normalize() * speed.0 * time.delta_seconds();
            }
        }
    }
}
