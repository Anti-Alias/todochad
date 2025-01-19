use bevy::prelude::*;
use bevy_mod_ui_dsl::*;
use crate::{GraphInfo, GuiAssets};
use crate::cursor::{pointer_on_over, default_on_out};
pub use action::*;

pub fn chad_ui_plugin(app: &mut App) {
    app.add_observer(spawn_left_panel);
    app.add_observer(spawn_right_panel);
    app.add_systems(Update, (render_left_panel, render_right_panel));
}

#[derive(Component, Debug)]
#[require(Node)]
pub struct LeftPanel {
    current_task: Option<tdc::TaskId>,
}

#[derive(Component, Debug)]
#[require(Node)]
pub struct RightPanel {
    todo_task_infos: Vec<TaskInfo>,
}

#[derive(Debug)]
struct TaskInfo {
    task_id: tdc::TaskId,
    doable: bool,
}

fn spawn_left_panel(_trigger: Trigger<action::SpawnLeftPanel>, mut commands: Commands) {
    commands.spawn((
        LeftPanel { current_task: Some(0) },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(300.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            ..default()
        }
    ));
}

fn render_left_panel(
    mut commands: Commands,
    mut left_panel_q: Query<(Entity, &LeftPanel), Changed<LeftPanel>>,
    gui_assets: Res<GuiAssets>,
) {
    let Some((panel_e, panel)) = left_panel_q.iter_mut().next() else { return };
    let s = &mut Spawner::relative(panel_e, &mut commands);
    let header_font = &gui_assets.ui_header_font;
    let font = &gui_assets.ui_font;

    NodeW::new().cfg(cfg::side_panel).begin(s);

        // Task group 
        if let Some(_task_id) = panel.current_task {
            TextW::new("Current Task").config(cfg::header, header_font).insert(s);
            NodeW::new().cfg(cfg::group).begin(s);
                NodeW::new().begin(s);
                    TextW::new("Name: ").config(cfg::text, font).insert(s);
                    TextW::new("Steve").config(cfg::text, font).insert(s);
                NodeW::end(s);
                NodeW::new().begin(s);
                    TextW::new("Created: ").config(cfg::text, font).insert(s);
                    TextW::new("<DATE>").config(cfg::text, font).insert(s);
                NodeW::end(s);
                NodeW::new().begin(s);
                    TextW::new("Updated: ").config(cfg::text, font).insert(s);
                    TextW::new("<DATE>").config(cfg::text, font).insert(s);
                NodeW::end(s);
            NodeW::end(s);
        }

        // Action group 
        TextW::new("Actions").config(cfg::header, header_font).insert(s);
        NodeW::new().cfg(cfg::group).begin(s);
            NodeW::new().begin(s);
                ButtonW::new().cfg(cfg::button_primary).begin(s);
                    let new_task_e = s.last();
                    TextW::new("New Task").config(cfg::text, font).insert(s);
                ButtonW::end(s);
                ButtonW::new().cfg(cfg::button_secondary).begin(s);
                    let save_e = s.last();
                    TextW::new("Save").config(cfg::text, font).insert(s);
                ButtonW::end(s);
            NodeW::end(s);
        NodeW::end(s);
    NodeW::end(s);

    // Callbacks
    commands
        .entity(new_task_e)
        .observe(pointer_on_over)
        .observe(default_on_out)
        .observe(new_task_on_press);
    commands
        .entity(save_e)
        .observe(pointer_on_over)
        .observe(default_on_out)
        .observe(save_on_press);
}

fn spawn_right_panel(
    _trigger: Trigger<action::SpawnRightPanel>,
    info: Res<GraphInfo>,
    mut commands: Commands,
) {
    let todo_task_infos = generate_task_infos(&info.graph);
    commands.spawn((
        RightPanel { todo_task_infos },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(300.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            ..default()
        }
    ));
}

fn render_right_panel(
    mut commands: Commands,
    mut right_panel_q: Query<(Entity, &RightPanel), Changed<RightPanel>>,
    gui_assets: Res<GuiAssets>,
    info: Res<GraphInfo>,
) {
    let Some((panel_e, panel)) = right_panel_q.iter_mut().next() else { return };
    let s = &mut Spawner::relative(panel_e, &mut commands);
    let header_font = &gui_assets.ui_header_font;
    let font = &gui_assets.ui_font;
    NodeW::new().cfg(cfg::side_panel).begin(s);

        TextW::new("Todo List").config(cfg::header, header_font).insert(s);
        NodeW::new().cfg(cfg::group).begin(s);
        for task_info in panel.todo_task_infos.iter() {
            let task = info.graph.get(task_info.task_id).unwrap();
            if task_info.doable {
                TextW::new(&task.name).config(cfg::todo_text, font).insert(s);
            }
            else {
                TextW::new(&task.name).config(cfg::todo_disabled_text, font).insert(s);
            }
        }
        NodeW::end(s);
    NodeW::end(s);
}


fn new_task_on_press(_trigger: Trigger<Pointer<Down>>) {
    // TODO
}

fn save_on_press(
    trigger: Trigger<Pointer<Down>>,
    info: Res<GraphInfo>,
) {
    let event = trigger.event();
    if event.button != PointerButton::Primary { return };
    let GraphInfo { config, graph } = &*info;
    graph.save(config).expect("Failed to save graph");
}

fn generate_task_infos(graph: &tdc::Graph) -> Vec<TaskInfo> {
    let mut task_infos: Vec<TaskInfo> = graph
        .traverse_selected()
        .into_iter()
        .map(|(task_id, task)| TaskInfo {
            task_id,
            doable: task.dependencies().is_empty(),
        })
        .collect();
    task_infos.sort_by_key(|task_info| !task_info.doable);
    task_infos
}

/// Actions that drive UI behavior.
mod action {
    use bevy::prelude::*;

    #[derive(Event, Debug)]
    pub struct SpawnLeftPanel;
    #[derive(Event, Debug)]
    pub struct SpawnRightPanel;
}

/// Config functions for widgets
mod cfg {
    use bevy::prelude::*;
    use bevy_mod_ui_dsl::*;

    pub fn side_panel(node: &mut NodeW) {
        node.node.flex_direction = FlexDirection::Column;
        node.node.align_items = AlignItems::Start;
        node.node.width = Val::Px(300.0);
        node.node.height = Val::Percent(100.0);
        node.background_color = Color::srgb(0.3, 0.3, 0.3).into();
    }

    pub fn group(node: &mut NodeW) {
        node.node.flex_direction = FlexDirection::Column;
        node.node.margin = UiRect::px(15.0, 0.0, 0.0, 10.0);
    }

    pub fn button_primary(button: &mut ButtonW) {
        button.node.justify_content = JustifyContent::Center;
        button.background_color = Color::srgb(0.1, 0.6, 0.2).into();
        button.node.padding = UiRect::all(Val::Px(5.0));
        button.node.margin = UiRect::right(Val::Px(5.0));
        button.border_radius = BorderRadius::all(Val::Px(3.0));
    }

    pub fn button_secondary(button: &mut ButtonW) {
        button.node.justify_content = JustifyContent::Center;
        button.background_color = Color::srgb(0.1, 0.2, 0.6).into();
        button.node.padding = UiRect::all(Val::Px(5.0));
        button.node.margin = UiRect::right(Val::Px(5.0));
        button.border_radius = BorderRadius::all(Val::Px(3.0));
    }

    pub fn text(text: &mut TextW, font: &TextFont) {
        text.text_font = font.clone();
    }

    pub fn todo_text(text: &mut TextW, font: &TextFont) {
        text.text_color = Color::srgba(1.0, 1.0, 0.5, 1.0).into();
        text.text_font = font.clone();
        text.node.margin = UiRect::px(0.0, 0.0, 5.0, 5.0);
    }

    pub fn todo_disabled_text(text: &mut TextW, font: &TextFont) {
        text.text_color = Color::srgba(1.0, 1.0, 1.0, 0.5).into();
        text.text_font = font.clone();
        text.node.margin = UiRect::px(0.0, 0.0, 5.0, 5.0);
    }

    pub fn header(text: &mut TextW, font: &TextFont) {
        text.text_font = font.clone();
        text.node.margin = UiRect::px(7.0, 0.0, 5.0, 5.0);
    }
}
