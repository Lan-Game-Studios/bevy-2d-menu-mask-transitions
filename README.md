# Bevy Menu Transitions Plugin

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Doc](https://docs.rs/bevy_2d_menu_mask_transition/badge.svg)](https://docs.rs/bevy_2d_menu_mask_transition)
[![Crate](https://img.shields.io/crates/v/bevy_2d_menu_mask_transition.svg)](https://crates.io/crates/bevy_2d_menu_mask_transition)
[![Build Status](https://github.com/Lan-Game-Studios/bevy-2d-menu-mask-transitions/actions/workflows/rust.yml/badge.svg)](https://github.com/Lan-Game-Studios/bevy-2d-menu-mask-transitions/actions/workflows/rust.yml)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-v0.18-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![dependency status](https://deps.rs/repo/github/Lan-Game-Studios/bevy-2d-menu-mask-transitions/status.svg)](https://deps.rs/repo/github/Lan-Game-Studios/bevy-2d-menu-mask-transitions)

![](https://github.com/Lan-Game-Studios/bevy-2d-menu-mask-transitions/blob/main/docs/example-basic-long.gif)

[![Discord](https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg)](https://discord.gg/JN5c3vrp) 

## TODOs

- [ ] fix first time loading bug
- [ ] reduce texture overhead by using single channel textures instead of RGBA channels

## Overview

This project is a Bevy plugin that enables smooth menu transitions in Bevy applications or games. It provides a way to create visually appealing transitions between different game states, such as transitioning from a menu screen to gameplay. The plugin takes a screenshot of the current frame, applies a transition effect using a shader and a mask texture, and transitions to the new state.

## Features

- **State-based Transitions:** Supports transitions between different states in your Bevy application, such as `Menu` and `InGame`.
- **Customizable Transition Effects:** Allows the use of custom masks to define the visual appearance of transitions.
- **Integration with Bevy's UI System:** The plugin integrates smoothly with Bevy's UI system, enabling the creation of UI elements that interact with the transition effects.

## Installation

To include this plugin in your Bevy project, add the following to your `Cargo.toml`:

```shell
cargo add bevy_2d_menu_mask_transition
```

## Usage

1. **Initialize the Plugin:**
   In your main file (e.g., `main.rs`), initialize the `MenuTransitionPlugin` by adding it to your Bevy app:

   ```rust
   use bevy::prelude::*;
   use bevy_2d_menu_mask_transition::{MenuTransitionPlugin, TriggerMenuTransition};

   fn main() {
       App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(MenuTransitionPlugin::<YourState>::default())
            .init_state::<YourState>()
            .run();
   }
   ```

   By default, transition masks preserve their aspect ratio using center-crop behavior. To stretch the mask to the full window, configure the plugin with `MaskStretchMode::Stretch`:

   ```rust
   use bevy::prelude::*;
   use bevy_2d_menu_mask_transition::{MaskStretchMode, MenuTransitionPlugin};

   App::new()
       .add_plugins(DefaultPlugins)
       .add_plugins(
           MenuTransitionPlugin::<YourState>::default()
               .with_mask_stretch_mode(MaskStretchMode::Stretch),
       );
   ```

   `MaskStretchMode::PreserveAspect` avoids distortion while still covering the full screen.

2. **Define Your Game States:**
   Create an enum to represent the various states in your game:

   > `Debug` is required, but only used for debug level logging

   ```rust
   #[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
   enum YourState {
       #[default]
       Menu,
       InGame,
   }
   ```

3. **Trigger Transitions:**
   You can trigger transitions between states using the `TriggerMenuTransition` message. In Bevy 0.18 this uses the message API (`MessageWriter`) rather than events. For example, this can be tied to a button press:

   ```rust
   #[derive(Component, Default)]
   struct Navigate(YourState);

   fn interact(
       mut query: Query<(&Interaction, &Navigate), With<Button>>,
       mut writer: MessageWriter<TriggerMenuTransition<YourState>>,
       asset_server: Res<AssetServer>,
   ) {
       for (interaction, navigate) in query.iter_mut() {
           if *interaction == Interaction::Pressed {
               writer.write(TriggerMenuTransition {
                   target_state: navigate.0,
                   duration: Duration::from_secs_f32(1.0),
                   mask: asset_server.load("path/to/mask.png"),
               });
           }
       }
   }
   ```

4. **Try the Preserve Aspect Example:**

   ```shell
   cargo run --example preserve_aspect
   ```

### Contribution

If you'd like to contribute to this plugin, feel free to submit issues or pull requests on the GitHub repository.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE.md) file for more details.

## Acknowledgements

- [Bevy](https://bevyengine.org/) - A data-driven game engine built in Rust.
- The Bevy community for providing a robust and flexible game development framework.


## Compatibility


| Version | Bevy Version |
|---------|--------------|
| 0.3.x   | 0.18         |
| 0.2.x   | 0.18         |

### Bevy 0.18 Notes

- This crate targets Bevy 0.18 APIs.
- Transition requests are sent with `MessageWriter<TriggerMenuTransition<_>>`.
- Your app state must be initialized with `.init_state::<YourState>()`.
- The plugin now ensures Bevy's `StatesPlugin` exists before initializing its internal transition state.

### 0.3.0 Migration Notes

- Default mask sampling changed from stretched UVs to aspect-preserving center-crop.
- To restore the 0.2.x stretched behavior, set `.with_mask_stretch_mode(MaskStretchMode::Stretch)` on `MenuTransitionPlugin`.

## Lan Game Studios

This crate is part of an effort to create a game studio. Check out
[Mega Giga Cookie Destoryer TD](https://store.steampowered.com/app/2283070/Mega_Giga_Cookie_Destroyer_TD/) or
the mission of [Lan Game Studios](https://langamestudios.com) if you like games or game development.
