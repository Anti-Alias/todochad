use bevy::prelude::*;
use bevy_mod_ui_dsl::*;

use crate::{GraphInfo, GuiAssets};

pub fn graph_ui_plugin(app: &mut App) {
    app.add_observer(spawn_left_panel);
    app.add_systems(Update, render_left_panel);
}

#[derive(Component, Debug)]
#[require(Node)]
pub struct LeftPanel {
    current_task: Option<tdc::TaskId>,
}

fn spawn_left_panel(_trigger: Trigger<event::SpawnLeftPanel>, mut commands: Commands) {
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
    NodeW::new().cfg(c_left_panel).begin(s);

        // Task group 
        if let Some(_task_id) = panel.current_task {
            NodeW::new().cfg(c_group).begin(s);
                TextW::new("Current Task").config(c_header, header_font).insert(s);
                NodeW::new().begin(s);
                    TextW::new("Name: ").config(c_text, font).insert(s);
                    TextW::new("Steve").config(c_text, font).insert(s);
                NodeW::end(s);
                NodeW::new().begin(s);
                    TextW::new("Created: ").config(c_text, font).insert(s);
                    TextW::new("<DATE>").config(c_text, font).insert(s);
                NodeW::end(s);
                NodeW::new().begin(s);
                    TextW::new("Updated: ").config(c_text, font).insert(s);
                    TextW::new("<DATE>").config(c_text, font).insert(s);
                NodeW::end(s);
            NodeW::end(s);
        }

        // Action group 
        NodeW::new().cfg(c_group).begin(s);
            TextW::new("Actions").config(c_header, header_font).insert(s);
            NodeW::new().begin(s);
                ButtonW::new().cfg(c_button_primary).begin(s);
                    let new_task_e = s.last();
                    TextW::new("New Task").config(c_text, font).insert(s);
                ButtonW::end(s);
                ButtonW::new().cfg(c_button_secondary).begin(s);
                    let save_e = s.last();
                    TextW::new("Save").config(c_text, font).insert(s);
                ButtonW::end(s);
            NodeW::end(s);
        NodeW::end(s);
    NodeW::end(s);

    // Callbacks
    commands.entity(new_task_e).observe(new_task_on_press);
    commands.entity(save_e).observe(save_on_press);
}

pub mod event {
    use bevy::prelude::*;

    #[derive(Event, Debug)]
    pub struct SpawnLeftPanel;
}

fn c_left_panel(node: &mut NodeW) {
    node.node.flex_direction = FlexDirection::Column;
    node.node.align_items = AlignItems::Start;
    node.node.padding = UiRect::px(10.0, 0.0, 5.0, 0.0);
    node.node.width = Val::Px(300.0);
    node.node.height = Val::Percent(100.0);
    node.background_color = Color::srgb(0.3, 0.3, 0.3).into();
}

fn c_group(node: &mut NodeW) {
    node.node.flex_direction = FlexDirection::Column;
    node.node.margin = UiRect::bottom(Val::Px(10.0));
}

fn c_button_primary(button: &mut ButtonW) {
    button.node.justify_content = JustifyContent::Center;
    button.background_color = Color::srgb(0.1, 0.6, 0.2).into();
    button.node.padding = UiRect::all(Val::Px(5.0));
    button.node.margin = UiRect::right(Val::Px(5.0));
    button.border_radius = BorderRadius::all(Val::Px(3.0));
}

fn c_button_secondary(button: &mut ButtonW) {
    button.node.justify_content = JustifyContent::Center;
    button.background_color = Color::srgb(0.1, 0.2, 0.6).into();
    button.node.padding = UiRect::all(Val::Px(5.0));
    button.node.margin = UiRect::right(Val::Px(5.0));
    button.border_radius = BorderRadius::all(Val::Px(3.0));
}

fn c_text(text: &mut TextW, font: &TextFont) {
    text.text_font = font.clone();
}

fn c_header(text: &mut TextW, font: &TextFont) {
    text.text_font = font.clone();
    text.node.margin = UiRect::bottom(Val::Px(5.0));
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
