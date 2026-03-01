mod constants;
mod components;
mod spline;
mod setup;
mod systems;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use setup::setup_solar_system;
use systems::{physics_system, input_system, update_ui_with_settings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Apollo 11 - Modular Bevy Simulation".into(),
                resolution: (1600.0, 900.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, (setup_solar_system, setup_ui))
        .add_systems(Update, (physics_system, input_system, update_ui_with_settings))
        .run();
}

fn setup_ui(mut commands: Commands) {
    use components::*;
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        ..default()
    }).with_children(|root| {
        root.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, ..default() }, ..default() }).with_children(|left| {
            left.spawn(TextBundle::from_section("APOLLO 11", TextStyle { font_size: 48.0, color: Color::WHITE, ..default() }));
            left.spawn(TextBundle::from_section("High Fidelity Modular Simulation", TextStyle { font_size: 18.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() }));
            left.spawn((TextBundle::from_section("Phase: Translunar Coast", TextStyle { font_size: 22.0, color: Color::srgb(0.0, 1.0, 0.8), ..default() }).with_style(Style { margin: UiRect::top(Val::Px(20.0)), ..default() }), PhaseText));
        });

        root.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexEnd, ..default() }, ..default() }).with_children(|right| {
            right.spawn(NodeBundle {
                style: Style { flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(20.0)), border: UiRect::all(Val::Px(1.0)), margin: UiRect::bottom(Val::Px(20.0)), ..default() },
                background_color: Color::srgba(0.0, 0.05, 0.1, 0.75).into(),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            }).with_children(|panel| {
                panel.spawn((TextBundle::from_section("T+ 00h 00m", TextStyle { font_size: 24.0, color: Color::WHITE, ..default() }), TimeText));
                panel.spawn((TextBundle::from_section("Velocity: 0.00 km/s", TextStyle { font_size: 20.0, color: Color::srgb(0.0, 1.0, 0.8), ..default() }).with_style(Style{ margin: UiRect::top(Val::Px(10.0)), ..default() }), VelText));
                panel.spawn((TextBundle::from_section("Alt (Earth): 0 km", TextStyle { font_size: 20.0, color: Color::srgb(0.0, 1.0, 0.8), ..default() }), AltEText));
                panel.spawn((TextBundle::from_section("Alt (Moon): 0 km", TextStyle { font_size: 20.0, color: Color::srgb(0.0, 1.0, 0.8), ..default() }), AltMText));
            });

            right.spawn(NodeBundle {
                style: Style { padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                background_color: Color::srgba(0.0, 0.05, 0.1, 0.75).into(),
                border_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            }).with_children(|panel| {
                panel.spawn((TextBundle::from_section("SPACE|T|ARROWS", TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }), ControlText));
            });
        });
    });
}
