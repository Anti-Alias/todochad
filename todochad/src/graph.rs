use crate::cursor::{pointer_on_over, default_on_out};
use rand::prelude::*;
use bevy::prelude::*;
use bevy::text::TextBounds;
use std::collections::HashMap;
use tdc::TaskId;
pub use action::*;

use crate::MainCamera;

const TASK_COLOR: Color             = Color::srgb(0.1, 0.3, 0.5);
const TASK_SELECTED_COLOR: Color    = Color::srgb(0.1, 0.6, 0.3);
const TASK_NODE_SIZE: Vec2          = Vec2::new(7.0 * GRID_SIZE, 2.0 * GRID_SIZE);
const LINE_COLOR: Color             = Color::srgb(0.8, 0.5, 0.2);
const GRID_COLOR: Color             = Color::srgba(1.0, 1.0, 1.0, 0.02);
const GRID_SIZE: f32                = 30.0;
const MIN_X: f32    = -500.0;
const MIN_Y: f32    = -500.0;
const MAX_X: f32    = 500.0;
const MAX_Y: f32    = 500.0;

#[derive(Debug)]
pub struct GraphPlugin {
    pub config: tdc::Config,
    pub graph: tdc::Graph,
}

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphInfo {
            config: self.config.clone(),
            graph: self.graph.clone(),
        });
        app.init_resource::<GuiAssets>();
        app.init_resource::<TaskMapping>();
        app.add_observer(spawn_graph);
        app.add_systems(Update, (
            sync_task_xy,
            (draw_arrows_between_nodes, draw_grid), 
        ));
    }
}

/// Resource that stores the app's graph.
#[derive(Resource)]
pub struct GraphInfo {
    pub config: tdc::Config,
    pub graph: tdc::Graph,
}

/// Component storing a reference to a task in the graph.
#[derive(Component, Copy, Clone, Eq, PartialEq, Default, Debug)]
#[require(Transform, InheritedVisibility)]
pub struct TaskNode { pub task_id: TaskId }

/// Resource that maps tasks in the graph with entities.
#[derive(Resource, Default, Debug)]
pub struct TaskMapping {
    task_to_entity: HashMap<TaskId, Entity>,
}

impl TaskMapping {
    pub fn insert(&mut self, task_id: TaskId, node_id: Entity) {
        self.task_to_entity.insert(task_id, node_id);
    }
    pub fn get_entity(&self, task_id: TaskId) -> Option<Entity> {
        self.task_to_entity.get(&task_id).copied()
    }
}

/// Stores assets for the entire UI
#[derive(Resource, Debug)]
pub struct GuiAssets {
    pub task_font: TextFont, 
    pub ui_font: TextFont, 
    pub ui_header_font: TextFont, 
}

impl FromWorld for GuiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let font = assets.load("fonts/0xProtoNerdFont-Regular.ttf");
        Self {
            task_font: TextFont { font: font.clone(), font_size: 12.0, ..default() },
            ui_font: TextFont { font: font.clone(), font_size: 12.0, ..default() },
            ui_header_font: TextFont { font: font.clone(), font_size: 20.0, ..default() },
        }
    }
}


/// Events that trigger graph behaviors in the application.
mod action {
    use bevy::prelude::*;

    #[derive(Event)]
    pub struct SpawnGraph;
} 

/// Spawns graph + tasks when triggered.
/// Used at application startup.
fn spawn_graph(
    _trigger: Trigger<SpawnGraph>,
    info: Res<GraphInfo>,
    gui_assets: Res<GuiAssets>,
    mut commands: Commands, 
) {
    let mut task_mapping = TaskMapping::default();
    let mut z = 0.0;
    for (task_id, task) in info.graph.iter() {
        let (x, y) = get_task_position(task.xy);
        let color = if !task.selected { TASK_COLOR } else { TASK_SELECTED_COLOR };
        let task_e = commands.spawn((
            Sprite::from_color(color, TASK_NODE_SIZE),
            TaskNode { task_id },
            Transform::from_xyz(x, y, z),
        )).with_child((
            Text2d(task.name.clone()),
            TextBounds::new(TASK_NODE_SIZE.x, TASK_NODE_SIZE.y),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0001)),
            gui_assets.task_font.clone(),
        ))
        .observe(translate_on_drag)
        .observe(round_on_drag_end)
        .observe(pointer_on_over)
        .observe(default_on_out)
        .id();
        task_mapping.insert(task_id, task_e);
        z += 1.0;
    }
    commands.insert_resource(task_mapping);
}

fn get_task_position(pos: Option<(f32, f32)>) -> (f32, f32) {
    match pos {
        Some((x, y)) => (x, y),
        None => {
            let mut rng = thread_rng();
            let x = rng.gen_range(MIN_X..MAX_X);
            let y = rng.gen_range(MIN_Y..MAX_Y);
            (x, y)
        }
    }
}

fn draw_arrows_between_nodes(
    task_nodes: Query<(&TaskNode, &Transform)>,
    task_mapping: Res<TaskMapping>,
    info: ResMut<GraphInfo>,
    mut draw: Gizmos,
) {
    let task_half_size = TASK_NODE_SIZE / 2.0;
    for (node, node_transf)  in &task_nodes {
        let task = info.graph.get(node.task_id).unwrap();
        let task_min = node_transf.translation.xy() - task_half_size; 
        let task_max = node_transf.translation.xy() + task_half_size; 
        for dep_task_id in task.dependencies() {
            let dep_task_entity = task_mapping.get_entity(*dep_task_id).unwrap();
            let (_dep_node, dep_node_transf) = task_nodes.get(dep_task_entity).unwrap();
            let dep_min = dep_node_transf.translation.xy() - task_half_size; 
            let dep_max = dep_node_transf.translation.xy() + task_half_size; 
            let line_start = node_transf.translation.xy();
            let line_end = dep_node_transf.translation.xy();
            let line_start = trim_line_on_box(line_end, line_start, task_min, task_max);
            let line_end = trim_line_on_box(line_start, line_end, dep_min, dep_max);
            draw.arrow_2d(line_start, line_end, LINE_COLOR);
        }
    }
}

fn draw_grid(
    camera: Single<(&Transform, &OrthographicProjection), With<MainCamera>>,
    mut draw: Gizmos,
) {
    let (cam_transf, cam_proj) = camera.into_inner();
    let cam_bottom_left = cam_proj.area.min + cam_transf.translation.xy();
    let cam_top_right = cam_proj.area.max + cam_transf.translation.xy();
    let start = (cam_bottom_left / GRID_SIZE).floor().as_ivec2();
    let end = (cam_top_right / GRID_SIZE).ceil().as_ivec2();
    for x in start.x..end.x {
        let start = IVec2::new(x, start.y).as_vec2() * GRID_SIZE;
        let end = IVec2::new(x, end.y).as_vec2() * GRID_SIZE;
        draw.line_2d(start, end, GRID_COLOR);
    }
    for y in start.y..end.y {
        let start = IVec2::new(start.x, y).as_vec2() * GRID_SIZE;
        let end = IVec2::new(end.x, y).as_vec2() * GRID_SIZE;
        draw.line_2d(start, end, GRID_COLOR);
    }
}

fn translate_on_drag(
    trigger: Trigger<Pointer<Drag>>,
    camera_q: Query<&OrthographicProjection, With<MainCamera>>,
    mut transf_q: Query<&mut Transform, Without<MainCamera>>,
) {
    let Ok(cam_proj) = camera_q.get_single() else { return };
    let (entity, event) = (trigger.entity(), trigger.event());
    if event.button != PointerButton::Primary { return };
    let mut transf = transf_q.get_mut(entity).unwrap();
    transf.translation.x += event.delta.x * cam_proj.scale;
    transf.translation.y -= event.delta.y * cam_proj.scale;
}

fn round_on_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut transf_q: Query<&mut Transform>,
) {
    let (entity, event) = (trigger.entity(), trigger.event());
    if event.button != PointerButton::Primary { return };
    let mut transf = transf_q.get_mut(entity).unwrap();
    let top_left = transf.translation.xy() - TASK_NODE_SIZE / 2.0;
    let top_left_rounded = (top_left / GRID_SIZE).round() * GRID_SIZE;
    let new_translation = top_left_rounded + TASK_NODE_SIZE / 2.0;
    transf.translation = new_translation.extend(0.0);
}

fn sync_task_xy(
    mut info: ResMut<GraphInfo>,
    mut tasks: Query<(&TaskNode, &mut Transform), Changed<Transform>>,
) {
    for (task_node, task_transf) in &mut tasks {
        let xyz = task_transf.translation;
        let task = info.graph.get_mut(task_node.task_id).unwrap();
        task.xy = Some((xyz.x, xyz.y));
    }
}

fn trim_line_on_box(a: Vec2, mut b: Vec2, box_min: Vec2, box_max: Vec2) -> Vec2 {
    if a.x < box_min.x && b.x > box_min.x { b.x = box_min.x; }
    if a.x > box_max.x && b.x < box_max.x { b.x = box_max.x; }
    if a.y < box_min.y && b.y > box_min.y { b.y = box_min.y; }
    if a.y > box_max.y && b.y < box_max.y { b.y = box_max.y; }
    b
}
