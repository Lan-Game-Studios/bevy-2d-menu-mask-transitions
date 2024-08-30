use std::time::Duration;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_2d_menu_mask_transition::{MenuTransitionPlugin, TriggerMenuTransition};

const MASKS: [&str; 8] = [
    "gradient001.webp",
    "gradient002.webp",
    "gradient003.webp",
    "gradient004.webp",
    "gradient005.webp",
    "gradient006.webp",
    "noise001.webp",
    "noise002.webp",
];

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

/// used to choose mask
#[derive(Component, Default)]
struct Mask(String);

#[derive(Resource)]
struct CurrentTransitionMask(String);

impl Default for CurrentTransitionMask {
    fn default() -> Self {
        Self(MASKS[0].into())
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MenuTransitionPlugin::<MyState>::default()))
        .init_state::<MyState>()
        .init_resource::<CurrentTransitionMask>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                interacte_mask,
                interacte_navigate,
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
    let text_style = TextStyle {
        font_size: 58.0,
        ..default()
    };
    let button_style = Style {
        padding: UiRect::axes(Val::Px(45.0), Val::Px(30.0)),
        ..default()
    };
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
    let small_text_style = TextStyle {
        font_size: 18.0,
        ..default()
    };
    let small_button_style = Style {
        padding: UiRect::axes(Val::Px(15.0), Val::Px(10.0)),
        width: Val::Px(270.0),
        ..default()
    };
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::RowReverse,
                position_type: PositionType::Absolute,
                min_width: Val::Percent(100.0),
                min_height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::End,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(50.0),
                ..default()
            },
            ..default()
        })
        .with_children(|wrapper| {
            for path in MASKS.iter() {
                wrapper
                    .spawn((
                        ButtonBundle {
                            style: small_button_style.clone(),
                            ..default()
                        },
                        Mask(path.to_string()),
                    ))
                    .with_children(|button| {
                        button.spawn(TextBundle::from_section(*path, small_text_style.clone()));
                    });
            }
        });
}

fn interacte_navigate(
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
    current_transition_mask: Res<CurrentTransitionMask>,
) {
    for (navigate, interaction, mut background_color, mut visibility) in query.iter_mut() {
        match (*interaction, navigate) {
            (Interaction::Pressed, navigate) => {
                writer.send(TriggerMenuTransition {
                    target_state: navigate.0,
                    duration: Duration::from_secs_f32(1.0),
                    mask: asset_server.load(&current_transition_mask.0),
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

fn interacte_mask(
    mut query: Query<(&Mask, &Interaction, &mut BackgroundColor), With<Button>>,
    mut current_transition_mask: ResMut<CurrentTransitionMask>,
) {
    for (mask, interaction, mut background_color) in query.iter_mut() {
        match (*interaction, mask) {
            (Interaction::Pressed, mask) => current_transition_mask.0 = mask.0.clone(),
            (Interaction::Hovered, _) => *background_color = tailwind::STONE_500.into(),
            (Interaction::None, mask) => {
                *background_color = if mask.0 == current_transition_mask.0 {
                    tailwind::STONE_400.into()
                } else {
                    tailwind::STONE_700.into()
                }
            }
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
