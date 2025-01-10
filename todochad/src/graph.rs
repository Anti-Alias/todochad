use std::collections::HashMap;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use tdc::TaskId;

const TASK_WIDTH: f32 = 17.0;
const TASK_HEIGHT: f32 = 25.0;
const MIN_X: f32 = -500.0;
const MIN_Y: f32 = -500.0;
const MAX_X: f32 = 500.0;
const MAX_Y: f32 = 500.0;

pub fn graph_plugin(app: &mut App) {
    app.init_resource::<GuiAssets>();
    app.init_resource::<TaskMapping>();
    app.add_observer(spawn_graph);
    app.add_systems(Update, (
        draw_arrows_between_nodes,
        handle_mouse_input,
    ));
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
    pub circle: Handle<Mesh>,
    pub task_color: Handle<ColorMaterial>,
    pub task_selected_color: Handle<ColorMaterial>,
    pub task_font: TextFont, 
}

impl FromWorld for GuiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let font = assets.load("fonts/0xProtoNerdFont-Regular.ttf");
        let task_font = TextFont { font, font_size: 16.0, ..default() };
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let circle = meshes.add(Rectangle::new(TASK_WIDTH, TASK_HEIGHT));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let task_color = materials.add(Color::linear_rgb(0.1, 0.1, 0.1));
        let task_selected_color = materials.add(Color::linear_rgb(0.05, 0.25, 0.05));
        Self { circle, task_color, task_selected_color, task_font }
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
    commands.spawn(Camera2d);

    // Spawns task nodes, and maps them to tasks in the graph
    let mut rng = thread_rng();
    for (task_id, task) in graph.iter() {
        let x: f32 = rng.gen_range(MIN_X..MAX_X);
        let y: f32 = rng.gen_range(MIN_Y..MAX_Y);
        let scale = Vec3::new(task.name.len() as f32, 1.0, 1.0);
        let color = match task.selected {
            false => gui_assets.task_color.clone(),
            true => gui_assets.task_selected_color.clone(),
        };
        let task_entity = commands.spawn((
            TaskNode { task_id },
            Transform::from_xyz(x, y, 0.0),
        )).with_children(|p| {
            p.spawn((
                Mesh2d(gui_assets.circle.clone()),
                MeshMaterial2d(color),
                Transform::from_scale(scale),
            ));
            p.spawn((
                Text2d(task.name.clone()),
                gui_assets.task_font.clone(),
            ));
        }).id();
        task_mapping.insert(task_id, task_entity);
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

fn handle_mouse_input(
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    // Gets cursor position
    let Some(window) = windows.iter().next() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let cursor_pos = Vec2::new(
        cursor_pos.x - window.width() / 2.0,
        -(cursor_pos.y - window.height() / 2.0),
    );
    println!("{cursor_pos:?}");
}
