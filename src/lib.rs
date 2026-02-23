use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy_app::{App, Last, Plugin};
use bevy_asset::{Asset, AssetServer, Assets, Handle, embedded_asset};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_log::prelude::debug;
use bevy_reflect::TypePath;
use bevy_render::{
    render_resource::AsBindGroup,
    view::screenshot::{Screenshot, ScreenshotCaptured},
};
use bevy_shader::ShaderRef;
use bevy_state::{
    app::{AppExtStates, StatesPlugin},
    prelude::in_state,
    state::{FreelyMutableState, NextState, States},
};
use bevy_time::{Time, Timer, TimerMode};
use bevy_ui::{FocusPolicy, Node, PositionType, Val, ZIndex};
use bevy_ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin};
use bevy_window::PrimaryWindow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaskStretchMode {
    #[default]
    PreserveAspect,
    Stretch,
}

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
#[derive(Message)]
pub struct TriggerMenuTransition<T: Transitionalbe> {
    pub target_state: T,
    pub duration: Duration,
    /// this needs to load fast, be careful
    /// with the texture size or preload the image
    /// in advance
    pub mask: Handle<Image>,
}

pub struct MenuTransitionPlugin<T: Transitionalbe> {
    mask_stretch_mode: MaskStretchMode,
    marker: PhantomData<T>,
}

#[derive(Resource, Clone, Copy)]
struct MenuTransitionSettings<T: Transitionalbe> {
    mask_stretch_mode: MaskStretchMode,
    marker: PhantomData<T>,
}

impl<T: Transitionalbe> Default for MenuTransitionPlugin<T> {
    fn default() -> Self {
        Self {
            mask_stretch_mode: MaskStretchMode::PreserveAspect,
            marker: PhantomData,
        }
    }
}

impl<T: Transitionalbe> MenuTransitionPlugin<T> {
    pub const fn with_mask_stretch_mode(mut self, mask_stretch_mode: MaskStretchMode) -> Self {
        self.mask_stretch_mode = mask_stretch_mode;
        self
    }
}

impl<T: Transitionalbe> Plugin for MenuTransitionPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.insert_resource(MenuTransitionSettings::<T> {
            mask_stretch_mode: self.mask_stretch_mode,
            marker: PhantomData,
        })
        .add_plugins(UiMaterialPlugin::<MenuTransitionMaterial>::default())
        .init_state::<TransitionState>()
        .add_message::<TriggerMenuTransition<T>>()
        .add_systems(
            Last,
            (
                idle::<T>.run_if(in_state(TransitionState::Idle)),
                create_material::<T>.run_if(in_state(TransitionState::TakingScreenshot)),
                wait_for_assets::<T>.run_if(in_state(TransitionState::LoadingMaskAndScreenshot)),
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
    mask_stretch_mode: MaskStretchMode,
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
    #[uniform(6)]
    pub preserve_mask_aspect: f32,
}

impl MenuTransitionMaterial {
    fn new(
        mask: Handle<Image>,
        previous_image: Handle<Image>,
        duration: Duration,
        mask_stretch_mode: MaskStretchMode,
        time: &Time,
    ) -> Self {
        Self {
            mask,
            previous_image,
            startup: time.elapsed_secs_wrapped(),
            duration: duration.as_secs_f32(),
            preserve_mask_aspect: match mask_stretch_mode {
                MaskStretchMode::Stretch => 0.0,
                MaskStretchMode::PreserveAspect => 1.0,
            },
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
    mut reader: MessageReader<TriggerMenuTransition<T>>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
    settings: Res<MenuTransitionSettings<T>>,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
) {
    let Some(event) = reader.read().next() else {
        return;
    };
    let Ok(window) = windows.single() else {
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
        mask_stretch_mode: settings.mask_stretch_mode,
    };
    commands.spawn(Screenshot::window(window)).observe(
        move |screenshot_captured: On<ScreenshotCaptured>| {
            let mut i = image_arc.lock().unwrap();
            *i = Some(screenshot_captured.image.clone());
        },
    );
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
        prepare_menu_shader.mask_stretch_mode,
        &time,
    );
    let ui_material = menu_transition_materials.add(material);

    commands.spawn((
        MaterialNode(ui_material),
        ZIndex(i32::MAX),
        FocusPolicy::Block,
        Node {
            position_type: PositionType::Absolute,
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            ..Default::default()
        },
    ));

    prepare_menu_shader.screenshot = Some(screenshot_handle);
    next_transition_state.set(TransitionState::LoadingMaskAndScreenshot);
}

fn wait_for_assets<T: Transitionalbe>(
    mut commands: Commands,
    prepare_menu_shader: Res<PrepareMenuShader<T>>,
    mut next_state: ResMut<NextState<T>>,
    query: Query<Entity, With<MaterialNode<MenuTransitionMaterial>>>,
    mut next_transition_state: ResMut<NextState<TransitionState>>,
    asset_server: Res<AssetServer>,
) {
    let is_mask_loaded = asset_server.is_loaded_with_dependencies(prepare_menu_shader.mask.id());
    debug!("Loaded Mask {:?}", is_mask_loaded);

    if is_mask_loaded {
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
        if despawn.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            next_transition_state.set(TransitionState::Idle);
            debug!("Despawn MenuTransitionMaterial");
        }
    }
}
