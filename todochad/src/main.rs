use bevy::prelude::*;
            

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GuiAssets>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    gui_assets: Res<GuiAssets>,
) {
    commands.spawn(Camera2d);

    // Spawns task entity
    commands.spawn((
        Mesh2d(gui_assets.circle.clone()),
        MeshMaterial2d(gui_assets.task_color.clone()),
        TaskNode,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}



/// Marker component for task nodes
#[derive(Component, Copy, Clone, Eq, PartialEq, Default, Debug)]
struct TaskNode;

/// Stores assets for the entire UI
#[derive(Resource, Debug)]
struct GuiAssets {
    circle: Handle<Mesh>,
    task_color: Handle<ColorMaterial>,
}

impl FromWorld for GuiAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let circle = meshes.add(Circle::new(50.0));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let task_color = materials.add(Color::linear_rgb(1.0, 0.0, 0.0));
        Self { circle, task_color }
    }
}

/////////// Application Events /////////// 
