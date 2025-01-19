use bevy::prelude::*;
use bevy::window::{PrimaryWindow, SystemCursorIcon};
use bevy::winit::cursor::CursorIcon;

/// Sets mouse cursor to "pointer" when hovering over an entity.
pub fn pointer_on_over(
    _trigger: Trigger<Pointer<Over>>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    commands
        .entity(window.into_inner())
        .insert(CursorIcon::System(SystemCursorIcon::Pointer));
}

/// Sets mouse cursor to "default" when hovering over an entity.
pub fn default_on_out(
    _trigger: Trigger<Pointer<Out>>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    commands
        .entity(window.into_inner())
        .insert(CursorIcon::System(SystemCursorIcon::Default));
}
