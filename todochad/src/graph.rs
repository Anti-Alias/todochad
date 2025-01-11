use std::collections::HashMap;
use rand::prelude::*;
use tdc::TaskId;
use bevy::{prelude::*, text::TextBounds};

use crate::{Draggable, MainCamera, Zoom};

const TASK_NODE_SIZE: Vec2 = Vec2::new(200.0, 50.0);
const MIN_X: f32 = -500.0;
const MIN_Y: f32 = -500.0;
const MAX_X: f32 = 500.0;
const MAX_Y: f32 = 500.0;

#[derive(Debug)]
pub struct GraphPlugin {
    pub graph: tdc::Graph,
}

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Graph(self.graph.clone()));
        app.init_resource::<GuiAssets>();
        app.init_resource::<TaskMapping>();
        app.add_observer(spawn_graph);
        app.add_systems(Update, draw_arrows_between_nodes);
    }
}

/// Resource that stores the app's graph.
#[derive(Resource, Deref, DerefMut)]
pub struct Graph(pub tdc::Graph);

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
    pub task_color: Color,
    pub task_selected_color: Color, 
    pub task_font: TextFont, 
}

impl FromWorld for GuiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let font = assets.load("fonts/0xProtoNerdFont-Regular.ttf");
        Self {
            task_color: Color::linear_rgb(0.1, 0.1, 0.1),
            task_selected_color: Color::linear_rgb(0.05, 0.25, 0.05),
            task_font: TextFont { font, font_size: 12.0, ..default() },
        }
    }
}


// Events that trigger graph behaviors in the application.
pub mod actions {
    use bevy::prelude::*;

    #[derive(Event)]
    pub struct SpawnGraph;
} 

/// Spawns graph + tasks when triggered.
/// Used at application startup.
fn spawn_graph(
    _event: Trigger<actions::SpawnGraph>,
    graph: Res<Graph>,
    mut commands: Commands, 
    gui_assets: Res<GuiAssets>,
) {
    let mut task_mapping = TaskMapping::default();
    commands.spawn((Camera2d, MainCamera, Draggable));

    // Spawns task nodes, and maps them to tasks in the graph
    let mut rng = thread_rng();
    let mut z = 0.0;
    for (task_id, task) in graph.iter() {
        let x: f32 = rng.gen_range(MIN_X..MAX_X);
        let y: f32 = rng.gen_range(MIN_Y..MAX_Y);
        let color = if !task.selected { gui_assets.task_color } else { gui_assets.task_selected_color };
        let task_entity = commands.spawn((
            Sprite::from_color(color, TASK_NODE_SIZE),
            TaskNode { task_id },
            Transform::from_xyz(x, y, z),
        )).with_child((
            Text2d(task.name.clone()),
            TextBounds::new(TASK_NODE_SIZE.x, TASK_NODE_SIZE.y),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0001)),
            gui_assets.task_font.clone(),
        ))
        .observe(handle_dragging)
        .id();
        task_mapping.insert(task_id, task_entity);
        z += 10.0;
    }
    commands.insert_resource(Graph(graph.clone()));
    commands.insert_resource(task_mapping);
}

fn draw_arrows_between_nodes(
    task_nodes: Query<(&TaskNode, &Transform)>,
    task_mapping: Res<TaskMapping>,
    graph: ResMut<Graph>,
    mut draw: Gizmos,
) {
    for (node, node_transf)  in &task_nodes {
        let task = graph.get(node.task_id).unwrap();
        for dep_task_id in task.dependencies() {
            let dep_task_entity = task_mapping.get_entity(*dep_task_id).unwrap();
            let (_dep_node, dep_node_transf) = task_nodes.get(dep_task_entity).unwrap();
            draw.arrow_2d(
                node_transf.translation.xy(), 
                dep_node_transf.translation.xy(),
                Color::linear_rgb(0.0, 0.0, 0.0),
            );
        }
    }
}

fn handle_dragging(
    trigger: Trigger<Pointer<Drag>>,
    mut transf_q: Query<&mut Transform>,
    zoom: Res<Zoom>,
) {
    let (entity, event) = (trigger.entity(), trigger.event());
    if event.button != PointerButton::Primary { return };
    let mut transf = transf_q.get_mut(entity).unwrap();
    transf.translation.x += event.delta.x * zoom.scale(); 
    transf.translation.y -= event.delta.y * zoom.scale(); 
}
