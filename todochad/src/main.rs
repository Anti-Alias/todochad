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
        .run();
    Ok(())
}

fn startup(mut commands: Commands) {
    commands.trigger(SpawnGraph);
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

