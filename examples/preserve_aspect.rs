use std::time::Duration;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_2d_menu_mask_transition::{MaskStretchMode, MenuTransitionPlugin, TriggerMenuTransition};
use bevy_state::{
    app::AppExtStates,
    prelude::{State, States, state_changed},
};

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum MyState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component, Default)]
struct Navigate(MyState);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MenuTransitionPlugin::<MyState>::default()
                .with_mask_stretch_mode(MaskStretchMode::PreserveAspect),
        ))
        .insert_resource(ClearColor(tailwind::STONE_950.into()))
        .init_state::<MyState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
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
) {
    for (navigate, interaction, mut background_color, mut visibility) in query.iter_mut() {
        match (*interaction, navigate) {
            (Interaction::Pressed, navigate) => {
                writer.write(TriggerMenuTransition {
                    target_state: navigate.0,
                    duration: Duration::from_secs_f32(1.0),
                    mask: asset_server.load("gradient001.webp"),
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

fn update_background(my_state: Res<State<MyState>>, mut clear_color: ResMut<ClearColor>) {
    clear_color.0 = match my_state.get() {
        MyState::Menu => tailwind::STONE_950.into(),
        MyState::InGame => tailwind::VIOLET_500.into(),
    };
}
