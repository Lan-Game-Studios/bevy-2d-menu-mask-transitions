use std::time::Duration;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_2d_menu_mask_transition::{MenuTransitionPlugin, TriggerMenuTransition};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum MyState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component, Default)]
struct Marker;

/// used to make transitions
#[derive(Component, Default)]
struct Navigate(MyState);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MenuTransitionPlugin::<MyState>::default()))
        .init_state::<MyState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                interacte,
                update_background.run_if(state_changed::<MyState>),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(ColorMesh2dBundle {
        mesh: meshes.add(Circle::new(200.0)).into(),
        material: material.add(ColorMaterial::from_color(tailwind::BLUE_400)),
        ..default()
    });
    commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Circle::new(20_000.0)).into(),
            material: material.add(ColorMaterial::from_color(tailwind::VIOLET_500)),
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        },
        Marker,
    ));
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                grid_template_rows: vec![GridTrack::min_content()],
                grid_template_columns: vec![GridTrack::max_content(), GridTrack::max_content()],
                position_type: PositionType::Absolute,
                min_width: Val::Percent(100.0),
                min_height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                column_gap: Val::Px(50.0),
                ..default()
            },
            ..default()
        })
        .with_children(|wrapper| {
            let text_style = TextStyle {
                font_size: 58.0,
                ..default()
            };
            let button_style = Style {
                padding: UiRect::axes(Val::Px(45.0), Val::Px(30.0)),
                ..default()
            };
            wrapper
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        ..default()
                    },
                    Navigate(MyState::Menu),
                ))
                .with_children(|button| {
                    button.spawn(TextBundle::from_section("Go to Menu", text_style.clone()));
                });
            wrapper
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        ..default()
                    },
                    Navigate(MyState::InGame),
                ))
                .with_children(|button| {
                    button.spawn(TextBundle::from_section("Go to Game", text_style.clone()));
                });
        });
}

fn interacte(
    mut query: Query<
        (
            &Navigate,
            &Interaction,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        With<Button>,
    >,
    my_state: Res<State<MyState>>,
    mut writer: EventWriter<TriggerMenuTransition<MyState>>,
    asset_server: Res<AssetServer>,
) {
    for (navigate, interaction, mut background_color, mut visibility) in query.iter_mut() {
        match (*interaction, navigate.0) {
            (Interaction::Pressed, state) => {
                writer.send(TriggerMenuTransition {
                    target_state: state,
                    duration: Duration::from_secs_f32(1.0),
                    mask: asset_server.load("noise003.webp"),
                });
            }
            (Interaction::Hovered, _) => *background_color = tailwind::STONE_500.into(),
            (Interaction::None, _) => *background_color = tailwind::STONE_700.into(),
        }

        if &navigate.0 != my_state.get() {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_background(
    my_state: Res<State<MyState>>,
    mut query: Query<&mut Visibility, With<Marker>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = match my_state.get() {
            MyState::Menu => Visibility::Hidden,
            MyState::InGame => Visibility::Visible,
        }
    }
}
