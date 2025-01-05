use std::collections::HashMap;
use bevy::prelude::*;
use rand::prelude::*;
use thiserror::*;
use tdc::{ graph_path, TaskId, GraphError };
use std::fs;
use std::path::Path;

const TASK_WIDTH: f32 = 17.0;
const TASK_HEIGHT: f32 = 25.0;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let graph_path = graph_path()?;
    let graph = load_graph(&graph_path)?;
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GuiAssets>()
        .init_resource::<TaskMapping>()
        .insert_resource(Graph(graph))
        .add_systems(Startup, startup)
        .add_systems(Update, draw_arrows_between_nodes)
        .run();
    Ok(())
}

fn startup(
    mut commands: Commands, 
    mut task_mapping: ResMut<TaskMapping>,
    graph: Res<Graph>,
    gui_assets: Res<GuiAssets>
) {
    // Region tasks can spawn in randomly 
    const MIN_X: f32 = -500.0;
    const MIN_Y: f32 = -500.0;
    const MAX_X: f32 = 500.0;
    const MAX_Y: f32 = 500.0;
    let mut rng = thread_rng();

    // Spawns camera
    commands.spawn(Camera2d);

    // Spawns task nodes, and maps them to tasks in the graph
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

#[derive(Resource, Deref, DerefMut)]
struct Graph(tdc::Graph);

/// Marker component for task nodes
#[derive(Component, Copy, Clone, Eq, PartialEq, Default, Debug)]
#[require(Transform, InheritedVisibility)]
struct TaskNode {
    task_id: TaskId,
}

/// Resource that maps tasks in the graph with task node entities.
#[derive(Resource, Default, Debug)]
struct TaskMapping {
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
struct GuiAssets {
    circle: Handle<Mesh>,
    task_color: Handle<ColorMaterial>,
    task_selected_color: Handle<ColorMaterial>,
    task_font: TextFont, 
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

fn load_graph(graph_path: &Path) -> Result<tdc::Graph> {
    match fs::exists(graph_path) { 
        Err(_) => Err(AppError::GraphReadError),
        Ok(false) => Ok(tdc::Graph::new()),
        Ok(true) => {
            let graph_string = fs::read_to_string(graph_path).map_err(|_| AppError::GraphReadError)?;
            let graph = tdc::Graph::read_str(&graph_string)?;
            Ok(graph)
        },
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to read graph file")]
    GraphReadError,
    #[error("Failed to write graph file")]
    GraphWriteError,
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

type Result<T> = std::result::Result<T, AppError>;

/////////// Application Events /////////// 
