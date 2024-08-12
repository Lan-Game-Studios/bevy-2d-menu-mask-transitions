# Bevy Menu Transitions Plugin

## TODOs

- [ ] Cleanup README
    - [ ] add compatibility table
    - [ ] cleanup usage section
    - [ ] add gif of nice menu transition
- [ ] fix first time loading bug

## Overview

This project is a Bevy plugin that enables smooth menu transitions in Bevy applications or games. It provides a way to create visually appealing transitions between different game states, such as transitioning from a menu screen to gameplay. The plugin takes a screenshot of the current frame, applies a transition effect using a shader and a mask texture, and transitions to the new state.

## Features

- **State-based Transitions:** Supports transitions between different states in your Bevy application, such as `Menu` and `InGame`.
- **Customizable Transition Effects:** Allows the use of custom shaders and masks to define the visual appearance of transitions.
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
           .add_plugin(MenuTransitionPlugin::<YourState>::default())
           .run();
   }
   ```

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
   You can trigger transitions between states using the `TriggerMenuTransition` event. For example, this can be tied to a button press:

   ```rust
   #[derive(Component, Default)]
   struct Navigate(YourState);

   fn interact(
       mut query: Query<(&Interaction, &Navigate), With<Button>>,
       mut writer: EventWriter<TriggerMenuTransition<YourState>>,
       asset_server: Res<AssetServer>,
   ) {
       for (interaction, navigate) in query.iter_mut() {
           if *interaction == Interaction::Pressed {
               writer.send(TriggerMenuTransition {
                   target_state: navigate.0,
                   duration: Duration::from_secs_f32(1.0),
                   mask: asset_server.load("path/to/mask.png"),
               });
           }
       }
   }
   ```

### Example

**example/basic.rs**

// TODO add gif

### Contribution

If you'd like to contribute to this plugin, feel free to submit issues or pull requests on the GitHub repository.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE.md) file for more details.

## Acknowledgements

- [Bevy](https://bevyengine.org/) - A data-driven game engine built in Rust.
- The Bevy community for providing a robust and flexible game development framework.


