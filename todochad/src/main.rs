mod graph;
mod ui;
mod camera;
pub mod cursor;

use graph::*;
use ui::*;
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
            chad_ui_plugin,
        ))
        .add_systems(Startup, startup)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
    Ok(())
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera, Draggable));
    commands.trigger(SpawnGraph);
    commands.trigger(SpawnLeftPanel);
    commands.trigger(SpawnRightPanel);
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    ConfigError(#[from] tdc::ConfigError),
    #[error(transparent)]
    GraphError(#[from] tdc::GraphError),
}

type Result<T> = std::result::Result<T, AppError>;

