use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy_app::{App, Plugin, PreUpdate, Update};
use bevy_asset::{embedded_asset, Asset, AssetServer, Assets, Handle, LoadState};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    query::With,
    schedule::IntoSystemConfigs,
    system::Resource,
    system::{Commands, Query, Res, ResMut},
};
use bevy_hierarchy::prelude::DespawnRecursiveExt;
use bevy_log::prelude::debug;
use bevy_reflect::TypePath;
use bevy_render::{
    render_resource::{AsBindGroup, ShaderRef},
    texture::Image,
    view::screenshot::ScreenshotManager,
};
use bevy_state::{
    app::AppExtStates,
    prelude::in_state,
    state::{FreelyMutableState, NextState, States},
};
use bevy_time::{Time, Timer, TimerMode};
use bevy_ui::{
    node_bundles::MaterialNodeBundle, FocusPolicy, PositionType, Style, UiMaterial,
    UiMaterialPlugin, Val, ZIndex,
};
use bevy_utils::default;
use bevy_window::PrimaryWindow;

pub trait Transitionalbe:
    States + PartialEq + Eq + Clone + Hash + FreelyMutableState + Debug
{
}
impl<T> Transitionalbe for T where
    T: States + PartialEq + Eq + Clone + Hash + FreelyMutableState + Debug
{
}

/// transition to a new game state with a transition effect
/// a screenshot of the state before will be taken and applied
/// with a filter mask
#[derive(Event)]
pub struct TriggerMenuTransition<T: Transitionalbe> {
    pub target_state: T,
    pub duration: Duration,
    /// this needs to load fast, be careful
    /// with the texture size or preload the image
    /// in advance
    pub mask: Handle<Image>,
}

#[derive(Default)]
pub struct MenuTransitionPlugin<T: Transitionalbe>(PhantomData<T>);
impl<T: Transitionalbe> Plugin for MenuTransitionPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<MenuTransitionMaterial>::default())
            .init_state::<TransitionState>()
            .add_event::<TriggerMenuTransition<T>>()
            .add_systems(Update, despawn)
            .add_systems(
                PreUpdate,
                (
                    idle::<T>.run_if(in_state(TransitionState::Idle)),
                    create_material::<T>.run_if(in_state(TransitionState::TakingScreenshot)),
                    wait_for_assets::<T>
                        .run_if(in_state(TransitionState::LoadingMaskAndScreenshot)),
                    despawn.run_if(in_state(TransitionState::Transitioning)),
                ),
            );
        embedded_asset!(app, "transition.wgsl");
    }
}

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum TransitionState {
    #[default]
    Idle,
    TakingScreenshot,
    LoadingMaskAndScreenshot,
    Transitioning,
}

#[derive(Component, Default)]
struct Despawn(Timer);

#[derive(Resource)]
struct PrepareMenuShader<T: Transitionalbe> {
    image: Arc<Mutex<Option<Image>>>,
    target_state: T,
    duration: Duration,
    mask: Handle<Image>,
    screenshot: Option<Handle<Image>>,
}

#[derive(Debug, Clone, AsBindGroup, TypePath, Default, Asset)]
struct MenuTransitionMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mask: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub previous_image: Handle<Image>,
    #[uniform(4)]
    pub startup: f32,
    #[uniform(5)]
    pub duration: f32,
}

impl MenuTransitionMaterial {
    fn new(
        mask: Handle<Image>,
        previous_image: Handle<Image>,
        duration: Duration,
        time: &Time,
    ) -> Self {
        Self {
            mask,
            previous_image,
            startup: time.elapsed_seconds_wrapped(),
            duration: duration.as_secs_f32(),
        }
    }
}

impl UiMaterial for MenuTransitionMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("embedded://bevy_2d_menu_mask_transition/transition.wgsl".into())
    }
}

/// create a texture from the current frame and
/// store it for the shader to transition
fn idle<T: Transitionalbe>(
    mut reader: EventReader<TriggerMenuTransition<T>>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
) {
    let Some(event) = reader.read().next() else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };

    debug!("Preparing a menu transition shader");

    let image_arc = Arc::new(Mutex::new(None));
    let prepare = PrepareMenuShader {
        image: image_arc.clone(),
        target_state: event.target_state.clone(),
        duration: event.duration,
        mask: event.mask.clone(),
        screenshot: None,
    };
    let _ = screenshot_manager.take_screenshot(window, move |image| {
        let mut i = image_arc.lock().unwrap();
        *i = Some(image);
    });
    commands.insert_resource(prepare);
    next_transition_state.set(TransitionState::TakingScreenshot);
}

fn create_material<T: Transitionalbe>(
    mut commands: Commands,
    mut menu_transition_materials: ResMut<Assets<MenuTransitionMaterial>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut prepare_menu_shader: ResMut<PrepareMenuShader<T>>,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
) {
    let arc = prepare_menu_shader.image.clone();
    let mutex_guard = arc.try_lock();
    let Ok(Some(image)) = mutex_guard.as_deref() else {
        return;
    };
    debug!("Processing a menu transition shader");
    let screenshot_handle = images.add(image.clone());
    let material = MenuTransitionMaterial::new(
        prepare_menu_shader.mask.clone(),
        screenshot_handle.clone(),
        prepare_menu_shader.duration,
        &time,
    );
    let ui_material = menu_transition_materials.add(material);

    commands.spawn((MaterialNodeBundle {
        z_index: ZIndex::Global(i32::MAX),
        focus_policy: FocusPolicy::Block,
        style: Style {
            position_type: PositionType::Absolute,
            height: Val::Vh(100.),
            width: Val::Vw(100.),
            ..default()
        },
        material: ui_material,
        ..default()
    },));

    prepare_menu_shader.screenshot = Some(screenshot_handle);
    next_transition_state.set(TransitionState::LoadingMaskAndScreenshot);
}

fn wait_for_assets<T: Transitionalbe>(
    mut commands: Commands,
    prepare_menu_shader: Res<PrepareMenuShader<T>>,
    mut next_state: ResMut<NextState<T>>,
    query: Query<Entity, With<Handle<MenuTransitionMaterial>>>,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
    asset_server: Res<AssetServer>,
) {
    let mask_load_state = asset_server.load_state(&prepare_menu_shader.mask);
    debug!("Loaded Mask {:?}", mask_load_state);

    if mask_load_state == LoadState::Loaded {
        for entity in query.iter() {
            commands.entity(entity).insert(Despawn(Timer::new(
                prepare_menu_shader.duration,
                TimerMode::Once,
            )));
            debug!(
                "Added `Despawn` to {:?} with duration {:?}",
                entity, prepare_menu_shader.duration
            );
        }
        commands.remove_resource::<PrepareMenuShader<T>>();
        next_state.set(prepare_menu_shader.target_state.clone());
        debug!(
            "NextState applying {:?}",
            prepare_menu_shader.target_state.clone()
        );
        next_transition_state.set(TransitionState::Transitioning);
    }
}

fn despawn(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Despawn)>,
    mut commands: Commands,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        debug!(
            "Waiting to despawn delta {:?} remaing {:?}",
            time.delta(),
            despawn.0.remaining()
        );
        if despawn.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
            next_transition_state.set(TransitionState::Idle);
            debug!("Despawn MenuTransitionMaterial");
        }
    }
}
