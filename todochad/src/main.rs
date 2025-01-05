use todochad::*;
use todochad::actions::*;
use bevy::prelude::*;
use thiserror::*;
use tdc::{ graph_path, GraphError };
use std::fs;
use std::path::Path;


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
        .insert_resource(Graph(graph))
        .add_plugins((DefaultPlugins, graph_plugin))
        .add_systems(Startup, startup)
        .add_systems(Update, draw_arrows_between_nodes)
        .run();
    Ok(())
}

fn startup(mut commands: Commands) {
    commands.trigger(SpawnGraph);
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

