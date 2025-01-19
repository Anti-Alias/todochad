use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const CAM_SPEED: f32 = 1000.0;
const CAM_ZOOM_SPEED_KEYBOARD: f32 = 0.1;
const CAM_ZOOM_SPEED_WHEEL: f32 = 0.1;
const CAM_ZOOM_MIN: f32 = 0.5;
const CAM_ZOOM_MAX: f32 = 3.0;

pub fn camera_pan_plugin(app: &mut App) {
    app.init_resource::<Cursor>();
    app.add_systems(Update, 
        (
            read_cursor, 
            (drag_camera, control_camera),
        ).chain()
    );
}

/// Reads cursor information into the [`Cursor`] resource.
/// Selects / deselects draggable entities.
fn read_cursor(
    mut cursor: ResMut<Cursor>,
    input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(Entity, &mut Transform), (With<MainCamera>, With<Draggable>)>,
) {
    if let Some(win) = windows.iter().next() {
        if let Some(pos) = win.cursor_position() {
            cursor.position = Vec2::new(pos.x - win.width() / 2.0, -(pos.y - win.height() / 2.0));
        } 
    }
    cursor.left_just_pressed = input.just_pressed(MouseButton::Left);
    cursor.left_just_released = input.just_released(MouseButton::Left);
    cursor.middle_just_pressed = input.just_pressed(MouseButton::Middle);
    cursor.middle_just_released = input.just_released(MouseButton::Middle);
    cursor.right_just_pressed = input.just_pressed(MouseButton::Right);
    cursor.right_just_released = input.just_released(MouseButton::Right);

    // Starts dragging camera
    if cursor.middle_just_pressed && !cursor.is_dragging() {
        let Some((camera_e, camera_transf)) = camera_q.iter().next() else { return };
        cursor.drag_state = DragState::Dragging {
            entity: camera_e,
            cursor_press_position: cursor.position,
            entity_press_position: camera_transf.translation.xy(),
            reverse: true,
        };
    }

    // Stops dragging camera
    if cursor.middle_just_released && cursor.is_dragging() {
        cursor.drag_state = DragState::None;
    }
}

fn drag_camera(
    cursor: ResMut<Cursor>, 
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
) {
    let Ok((mut cam_transf, cam_proj)) = camera_q.get_single_mut() else { return };
    let DragState::Dragging { 
        cursor_press_position, 
        entity_press_position,
        reverse,
        ..
    } = cursor.drag_state else { return };

    let coef = if reverse { -1.0 } else { 1.0 };
    let cursor_translation = cursor.position - cursor_press_position;
    let translation = entity_press_position + cam_proj.scale * coef * cursor_translation;
    cam_transf.translation = translation.extend(0.0);
}

fn control_camera(
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    mut scroll_events: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Scales camera using scroll wheel
    let Ok((mut cam_transf, mut cam_proj)) = camera_q.get_single_mut() else { return };
    for scroll_event in scroll_events.read() {
        let MouseScrollUnit::Line = scroll_event.unit else { continue };
        cam_proj.scale -= CAM_ZOOM_SPEED_WHEEL * scroll_event.y;
    }

    // Scales camera using + and - keys
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl_pressed && keyboard.just_pressed(KeyCode::Equal) {
        cam_proj.scale -= CAM_ZOOM_SPEED_KEYBOARD; 
    }
    if ctrl_pressed && keyboard.just_pressed(KeyCode::Minus) {
        cam_proj.scale += CAM_ZOOM_SPEED_KEYBOARD; 
    }
    if ctrl_pressed && keyboard.just_pressed(KeyCode::Digit0) {
        cam_proj.scale = 1.0;
    }
    cam_proj.scale = cam_proj.scale.clamp(CAM_ZOOM_MIN, CAM_ZOOM_MAX);

    // Moves camera using arrow keys
    let cam_speed = CAM_SPEED * cam_proj.scale;
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) { 
        cam_transf.translation.x += cam_speed * time.delta_secs();
    }

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) { 
        cam_transf.translation.x -= cam_speed * time.delta_secs();
    }

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) { 
        cam_transf.translation.y += cam_speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) { 
        cam_transf.translation.y -= cam_speed * time.delta_secs();
    }
}

#[derive(Resource, Default, Debug)]
pub struct Cursor {
   pub position: Vec2,
   pub left_just_pressed: bool,
   pub left_just_released: bool,
   pub middle_just_pressed: bool,
   pub middle_just_released: bool,
   pub right_just_pressed: bool,
   pub right_just_released: bool,
   pub drag_state: DragState,
}

impl Cursor {
    pub fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragState::Dragging { .. } => true,
            DragState::None => false,
        }
    }
}

#[derive(Component, Debug)]
pub struct MainCamera;

#[derive(PartialEq, Default, Debug)]
pub enum DragState {
    #[default]
    None,
    Dragging { 
        entity: Entity,
        cursor_press_position: Vec2,
        entity_press_position: Vec2,
        reverse: bool,
    }
}

#[derive(Component, Debug)]
pub struct Draggable;

