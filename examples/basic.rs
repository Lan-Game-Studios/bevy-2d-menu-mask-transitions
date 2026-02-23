use std::time::Duration;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_2d_menu_mask_transition::{MenuTransitionPlugin, TriggerMenuTransition};
use bevy_state::{
    app::AppExtStates,
    prelude::{State, States, state_changed},
};

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
        .insert_resource(ClearColor(tailwind::STONE_950.into()))
        .init_state::<MyState>()
        .init_resource::<CurrentTransitionMask>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                interact_mask,
                interact_navigate,
                update_background.run_if(state_changed::<MyState>),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|wrapper| {
            spawn_nav_button(wrapper, "Go to Menu", MyState::Menu);
            spawn_nav_button(wrapper, "Go to Game", MyState::InGame);
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                right: Val::Px(24.0),
                bottom: Val::Px(24.0),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|wrapper| {
            for path in MASKS {
                wrapper
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::WHITE),
                        BackgroundColor(tailwind::STONE_700.into()),
                        Mask(path.to_string()),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(path),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn spawn_nav_button(parent: &mut ChildSpawnerCommands, label: &str, state: MyState) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(45.0), Val::Px(30.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(tailwind::STONE_700.into()),
            Navigate(state),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn interact_navigate(
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
    mut writer: MessageWriter<TriggerMenuTransition<MyState>>,
    asset_server: Res<AssetServer>,
    current_transition_mask: Res<CurrentTransitionMask>,
) {
    for (navigate, interaction, mut background_color, mut visibility) in query.iter_mut() {
        match (*interaction, navigate) {
            (Interaction::Pressed, navigate) => {
                writer.write(TriggerMenuTransition {
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

fn interact_mask(
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

fn update_background(my_state: Res<State<MyState>>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = match my_state.get() {
        MyState::Menu => tailwind::STONE_950.into(),
        MyState::InGame => tailwind::VIOLET_500.into(),
    };
}
