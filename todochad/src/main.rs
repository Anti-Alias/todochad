mod graph;
mod ui;
mod camera;

use graph::*;
use graph::event::*;
use ui::*;
use ui::event::*;
use camera::*;

use bevy::prelude::*;
use thiserror::*;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = tdc::Config::load()?;
    let graph = tdc::Graph::load(&config)?;
    App::new()
        .add_plugins((
            DefaultPlugins, 
            GraphPlugin { config, graph },
            camera_pan_plugin,
            graph_ui_plugin,
        ))
        .add_systems(Startup, startup)
        .run();
    Ok(())
}

fn startup(mut commands: Commands) {
    commands.trigger(SpawnGraph);
    commands.trigger(SpawnLeftPanel);
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] tdc::ConfigError),
    #[error(transparent)]
    GraphError(#[from] tdc::GraphError),
}

type Result<T> = std::result::Result<T, AppError>;

