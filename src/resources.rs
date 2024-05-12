use bevy::prelude::*;

pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseCoords>()
            .init_resource::<BoxCoords>()
            // .init_resource::<DragSelect>();
            .init_resource::<GameCommands>();
    }
}

#[derive(Resource, Default, Debug)]
pub struct MouseCoords {
    pub global: Vec3,
    pub local: Vec2,
}

#[derive(Resource, Default, Debug)]
pub struct BoxCoords {
    pub global_start: Vec3,
    pub global_end: Vec3,
    pub local_start: Vec2,
    pub local_end: Vec2,
}

impl BoxCoords {
    // pub fn empty_local(&mut self) {
    //     self.local_start = Vec2::ZERO;
    //     self.local_end = Vec2::ZERO;
    // }

    pub fn empty_global(&mut self) {
        self.global_start = Vec3::ZERO;
        self.global_end = Vec3::ZERO;
    }
}

// #[derive(Resource, Default)]
// pub struct DragSelect(pub bool);

#[derive(Resource, Default)]
pub struct GameCommands {
    pub drag_select: bool,
}
