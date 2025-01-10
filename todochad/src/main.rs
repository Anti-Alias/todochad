use todochad::*;
use todochad::actions::*;
use bevy::prelude::*;
use thiserror::*;
use tdc::{ Config, ConfigError, Graph, GraphError };

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::load()?;
    let graph = Graph::load(&config)?;
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

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

type Result<T> = std::result::Result<T, AppError>;

