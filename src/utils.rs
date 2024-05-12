use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext, render::ColliderDebugColor};
use bevy_rts_camera::RtsCamera;

use crate::{
    resources::{BoxCoords, GameCommands, MouseCoords},
    MapBase, Selected, Unit,
};

pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_box_coords,
                set_mouse_coords,
                set_drag_select,
                drag_select,
                single_select,
                deselect_all,
            ),
        );
    }
}

fn set_drag_select(box_coords: Res<BoxCoords>, mut game_cmds: ResMut<GameCommands>) {
    let drag_threshold = 2.5;
    let width_z = (box_coords.global_start.z - box_coords.global_end.z).abs();
    let width_x = (box_coords.global_start.x - box_coords.global_end.x).abs();

    if width_z > drag_threshold || width_x > drag_threshold {
        game_cmds.drag_select = true;
    }
}

fn set_box_coords(
    mut box_coords: ResMut<BoxCoords>,
    mut game_cmds: ResMut<GameCommands>,
    input: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseCoords>,
) {
    if input.just_pressed(MouseButton::Left) {
        box_coords.global_start = mouse_coords.global;
        box_coords.local_start = mouse_coords.local;
    }

    if input.pressed(MouseButton::Left) {
        box_coords.local_end = mouse_coords.local;
        box_coords.global_end = mouse_coords.global;
    }

    if input.just_released(MouseButton::Left) {
        box_coords.empty_global();
        game_cmds.drag_select = false;
    }
}

// referenced https://bevy-cheatbook.github.io/cookbook/cursor2world.html
fn set_mouse_coords(
    mut mouse_coords: ResMut<MouseCoords>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    map_base_q: Query<&GlobalTransform, With<MapBase>>,
) {
    let (cam, cam_trans) = cam_q.single();
    let map_base_trans = map_base_q.single();
    let window = window_q.single();
    let Some(local_cursor) = window.cursor_position() else {
        return;
    };

    let plane_origin = map_base_trans.translation();
    let plane = Plane3d::new(map_base_trans.up());
    let Some(ray) = cam.viewport_to_world(cam_trans, local_cursor) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        return;
    };
    let global_cursor = ray.get_point(distance);

    mouse_coords.global = global_cursor;
    mouse_coords.local = local_cursor;
}

pub fn drag_select(
    mut cmds: Commands,
    mut gizmos: Gizmos,
    unit_q: Query<(Entity, &Transform), With<Unit>>,
    box_coords: Res<BoxCoords>,
    game_cmds: Res<GameCommands>,
) {
    if !game_cmds.drag_select {
        return;
    }

    let start = box_coords.global_start;
    let end = box_coords.global_end;

    // draw rectangle
    gizmos.line(start, Vec3::new(end.x, 0.0, start.z), Color::GRAY);
    gizmos.line(start, Vec3::new(start.x, 0.0, end.z), Color::GRAY);
    gizmos.line(Vec3::new(start.x, 0.0, end.z), end, Color::GRAY);
    gizmos.line(Vec3::new(end.x, 0.0, start.z), end, Color::GRAY);

    let min_x = start.x.min(end.x);
    let max_x = start.x.max(end.x);
    let min_z = start.z.min(end.z);
    let max_z = start.z.max(end.z);

    for (unit_ent, unit_trans) in unit_q.iter() {
        // check to see if units are within selection rectangle
        let unit_pos = unit_trans.translation;
        let in_box_bounds = unit_pos.x >= min_x
            && unit_pos.x <= max_x
            && unit_pos.z >= min_z
            && unit_pos.z <= max_z;

        if in_box_bounds {
            cmds.entity(unit_ent)
                .insert((ColliderDebugColor(Color::GREEN), Selected));
        } else {
            cmds.entity(unit_ent)
                .remove::<Selected>()
                .insert(ColliderDebugColor(Color::NONE));
        }
    }
}

pub fn single_select(
    mut cmds: Commands,
    rapier_context: Res<RapierContext>,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    select_q: Query<(Entity, &Selected)>,
    mouse_coords: Res<MouseCoords>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let (cam, cam_trans) = cam_q.single();

    let Some(ray) = cam.viewport_to_world(cam_trans, mouse_coords.local) else {
        return;
    };

    let hit = rapier_context.cast_ray(
        ray.origin,
        ray.direction.into(),
        f32::MAX,
        true,
        QueryFilter::only_dynamic(),
    );

    if let Some((ent, _toi)) = hit {
        // deselect all currently selected entities
        for (selected_entity, _) in select_q.iter() {
            cmds.entity(selected_entity)
                .insert(ColliderDebugColor(Color::NONE))
                .remove::<Selected>();
        }

        // select unit
        if !select_q.contains(ent) {
            cmds.entity(ent)
                .insert((ColliderDebugColor(Color::GREEN), Selected));
        }
    }
}

pub fn deselect_all(
    mut cmds: Commands,
    mut select_q: Query<Entity, With<Selected>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.just_pressed(MouseButton::Right) {
        for entity in select_q.iter_mut() {
            println!("Unit deselected");
            cmds.entity(entity).insert(ColliderDebugColor(Color::NONE));
            cmds.entity(entity).remove::<Selected>();
        }
    }
}
