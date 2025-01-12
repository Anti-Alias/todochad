use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const CAM_SPEED: f32 = 10.0;
const CAM_ZOOM_SPEED_KEYBOARD: f32 = 0.1;
const CAM_ZOOM_MIN: f32 = 0.5;
const CAM_ZOOM_MAX: f32 = 3.0;

pub fn camera_pan_plugin(app: &mut App) {
    app.init_resource::<Cursor>();
    app.init_resource::<Zoom>();
    app.add_systems(Update, 
        (
            read_cursor, 
            (drag_entity, control_camera),
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

fn drag_entity(
    cursor: ResMut<Cursor>,
    mut draggable_q: Query<&mut Transform>,
    zoom: Res<Zoom>,
) {
    let DragState::Dragging { 
        entity, 
        cursor_press_position, 
        entity_press_position,
        reverse,
    } = cursor.drag_state else { return };

    let coef = if reverse { -1.0 } else { 1.0 };
    let Ok(mut transf) = draggable_q.get_mut(entity) else { return };
    let cursor_translation = cursor.position - cursor_press_position;
    let translation = entity_press_position + zoom.scale() * coef * cursor_translation;
    transf.translation = Vec3::new(translation.x, translation.y, 0.0);
}

fn control_camera(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut zoom: ResMut<Zoom>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Scales camera using scroll wheel
    let Ok(mut cam_transf) = camera_q.get_single_mut() else { return };
    for scroll_event in scroll_events.read() {
        let MouseScrollUnit::Line = scroll_event.unit else { continue };
        zoom.0 -= scroll_event.y * 0.1;
    }

    // Scales camera using + and - keys
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl_pressed && keyboard.just_pressed(KeyCode::Equal) {
        zoom.0 -= CAM_ZOOM_SPEED_KEYBOARD; 
    }
    if ctrl_pressed && keyboard.just_pressed(KeyCode::Minus) {
        zoom.0 += CAM_ZOOM_SPEED_KEYBOARD; 
    }

    // Applies zoom to camera scaling
    zoom.0 = zoom.0.clamp(CAM_ZOOM_MIN, CAM_ZOOM_MAX);
    cam_transf.scale = Vec3::splat(zoom.scale());

    // Moves camera using arrow keys
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) { 
        cam_transf.translation.x += CAM_SPEED;
    }

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) { 
        cam_transf.translation.x -= CAM_SPEED;
    }

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) { 
        cam_transf.translation.y += CAM_SPEED;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) { 
        cam_transf.translation.y -= CAM_SPEED;
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

#[derive(Resource, Debug)]
pub struct Zoom(pub f32);
impl Zoom {
    pub fn scale(&self) -> f32 {
        self.0 * self.0
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self(1.0)
    }
}
